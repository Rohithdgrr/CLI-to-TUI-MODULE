use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Tui, attributes(tui, arg))]
pub fn derive_tui(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    
    // Parse struct-level #[tui(title = "...", description = "...")]
    let mut tui_title = struct_name.to_string();
    let mut tui_desc = None::<String>;
    
    for attr in &input.attrs {
        if attr.path().is_ident("tui") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("title") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    tui_title = value.value();
                    Ok(())
                } else if meta.path.is_ident("description") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    tui_desc = Some(value.value());
                    Ok(())
                } else {
                    Ok(())
                }
            }).ok();
        }
    }

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Tui only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "Tui only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut field_builders = Vec::new();
    let mut field_idents = Vec::new();
    
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_ident = field.ident.as_ref().unwrap();
        field_idents.push(field_ident);
        let rust_field_name = format_ident!("{}", field_name);
        
        // Determine type and widget
        let (widget_kind, is_option) = detect_widget(&field.ty);
        
        // Parse #[arg(short, long, default_value_t = ...)] 
        // and #[tui(label = "...", section = "...")]
        let mut label = field_name.replace("_", " ");
        let mut section = None::<String>;
        let mut required = true;
        let mut default_expr = None::<proc_macro2::TokenStream>;
        
        for attr in &field.attrs {
            if attr.path().is_ident("arg") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default_value_t") {
                        let value = meta.value()?;
                        let expr: syn::Expr = value.parse()?;
                        default_expr = Some(quote! { Some(tui_generator_core::value::Value::from(#expr)) });
                        required = false;
                        Ok(())
                    } else {
                        Ok(())
                    }
                }).ok();
            }
            if attr.path().is_ident("tui") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("label") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        label = value.value();
                        Ok(())
                    } else if meta.path.is_ident("section") {
                        let value: syn::LitStr = meta.value()?.parse()?;
                        section = Some(value.value());
                        Ok(())
                    } else {
                        Ok(())
                    }
                }).ok();
            }
        }

        if is_option {
            required = false;
        }
        
        let default_tokens = default_expr.unwrap_or_else(|| quote! { None });
        let section_tokens = match section {
            Some(s) => quote! { Some(#s.to_string()) },
            None => quote! { None },
        };

        field_builders.push(quote! {
            tui_generator_core::schema::Field {
                name: stringify!(#rust_field_name).to_string(),
                label: #label.to_string(),
                description: None,
                required: #required,
                default: #default_tokens,
                widget: #widget_kind,
                constraints: vec![],
                options: vec![],
                skip: false,
                section: #section_tokens,
                readonly: false,
            }
        });
    }

    let desc_tokens = match tui_desc {
        Some(d) => quote! { Some(#d.to_string()) },
        None => quote! { None },
    };

    let expanded = quote! {
        impl tui_generator_core::tui_trait::Tui for #struct_name {
            fn tui_schema() -> tui_generator_core::schema::TuiSchema {
                use tui_generator_core::widget::WidgetKind;
                tui_generator_core::schema::TuiSchema {
                    name: #tui_title.to_string(),
                    description: #desc_tokens,
                    fields: vec![#(#field_builders),*],
                    subcommands: vec![],
                }
            }

            fn parse_or_tui() -> Result<Self, tui_generator_core::error::TuiError> {
                use clap::Parser;
                let raw_args: Vec<String> = std::env::args().collect();
                let has_tui_flag = raw_args.contains(&"--tui".to_string());
                let clean_args: Vec<String> = raw_args.into_iter().filter(|a| a != "--tui").collect();

                // If explicitly requested via --tui or no CLI args provided, launch TUI
                if has_tui_flag || clean_args.len() <= 1 {
                    let schema = Self::tui_schema();
                    let state = tui_generator_ratatui::RatatuiRenderer::new().run_tui(&schema)?;
                    return Ok(Self::from_form_state(&state));
                }

                // Try normal CLI parse first
                match Self::try_parse_from(&clean_args) {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => {
                        use clap::error::ErrorKind;
                        match err.kind() {
                            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                                err.exit();
                            }
                            _ => {
                                // Fall back to TUI if CLI arguments are invalid or missing
                                let schema = Self::tui_schema();
                                let state = tui_generator_ratatui::RatatuiRenderer::new().run_tui(&schema)?;
                                Ok(Self::from_form_state(&state))
                            }
                        }
                    }
                }
            }
        }

        impl #struct_name {
            pub fn from_form_state(state: &tui_generator_core::state::FormState) -> Self {
                Self {
                    #(#field_idents: {
                        state.get_value(stringify!(#field_idents))
                            .cloned()
                            .and_then(|v| v.into())
                            .unwrap_or_default()
                    },)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn detect_widget(ty: &Type) -> (proc_macro2::TokenStream, bool) {
    let type_str = quote!(#ty).to_string().replace(" ", "");
    
    let is_option = type_str.starts_with("Option<");
    
    if type_str.contains("bool") {
        (quote! { WidgetKind::Checkbox }, is_option)
    } else if type_str.contains("u") || type_str.contains("i") || type_str.contains("f") {
        (quote! { WidgetKind::NumberInput }, is_option)
    } else if type_str.contains("PathBuf") {
        (quote! { WidgetKind::PathInput }, is_option)
    } else {
        (quote! { WidgetKind::TextInput }, is_option)
    }
}