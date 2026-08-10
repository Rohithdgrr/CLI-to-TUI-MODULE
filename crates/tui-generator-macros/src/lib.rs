use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Lit, Meta};

#[proc_macro_derive(Tui, attributes(tui, arg))]
pub fn derive_tui(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_tui(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_tui(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(f) => &f.named,
            _ => return Err(syn::Error::new_spanned(input, "Tui only supports structs with named fields")),
        },
        _ => return Err(syn::Error::new_spanned(input, "Tui only supports structs")),
    };

    let (title, description) = extract_struct_attrs(input);
    let mut field_defs = Vec::new();
    let mut from_value_arms = Vec::new();
    let mut to_value_arms = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let ty = &field.ty;
        let ty_str = type_to_string(ty);

        let label = extract_label(field).unwrap_or_else(|| to_title_case(&field_name_str));
        let doc = extract_doc_comment(field);
        let doc_ts = match &doc {
            Some(d) => quote! { Some(#d.to_string()) },
            None => quote! { None },
        };
        let (widget_ts, is_option) = determine_widget(field, &ty_str);
        let (default_ts, required) = extract_default(field, &ty_str, is_option);
        let options = extract_options(field);
        let skip = extract_skip(field);

        let options_tokens: Vec<proc_macro2::TokenStream> = options.iter()
            .map(|o| quote! { #o.to_string() })
            .collect();

        field_defs.push(quote! {
            ::tui_generator::core::schema::Field {
                name: #field_name_str.to_string(),
                label: #label.to_string(),
                description: #doc_ts,
                required: #required,
                default: #default_ts,
                widget: #widget_ts,
                constraints: vec![],
                options: vec![#(#options_tokens),*],
                skip: #skip,
            }
        });

        from_value_arms.push(gen_from_arm(field_name, &ty_str, is_option, skip));
        if !skip {
            to_value_arms.push(gen_to_arm(field_name, &ty_str));
        }
    }

    let title_str = title.unwrap_or_else(|| name.to_string());
    let desc_ts = description
        .map(|d| quote! { Some(#d.to_string()) })
        .unwrap_or_else(|| quote! { None });

    Ok(quote! {
        impl ::tui_generator::core::tui_trait::Tui for #name {
            fn schema() -> ::tui_generator::core::schema::TuiSchema {
                ::tui_generator::core::schema::TuiSchema {
                    name: #title_str.to_string(),
                    description: #desc_ts,
                    fields: vec![#(#field_defs),*],
                    subcommands: vec![],
                }
            }

            fn from_values(values: &std::collections::HashMap<String, ::tui_generator::core::value::Value>) -> Result<Self, ::tui_generator::core::error::TuiError> {
                Ok(#name { #(#from_value_arms),* })
            }

            fn to_values(&self) -> std::collections::HashMap<String, ::tui_generator::core::value::Value> {
                let mut map = std::collections::HashMap::new();
                #(#to_value_arms)*
                map
            }
        }

        impl #name {
            pub fn parse_or_tui() -> Result<Self, ::tui_generator::core::error::TuiError> {
                let args: Vec<String> = std::env::args().skip(1).collect();
                if args.iter().any(|a| a == "--tui") {
                    Self::run_tui()
                } else {
                    Ok(<Self as ::tui_generator::clap::Parser>::parse())
                }
            }

            pub fn run_tui() -> Result<Self, ::tui_generator::core::error::TuiError> {
                let schema = Self::schema();
                let state = ::tui_generator::ratatui::RatatuiRenderer::run(&schema)?;
                Self::from_values(&state.values)
            }
        }
    })
}

// --- Attribute parsing helpers (syn 2 API) ---

fn parse_tui_attrs(attr: &syn::Attribute) -> Vec<syn::Meta> {
    let mut result = Vec::new();
    if !attr.path().is_ident("tui") && !attr.path().is_ident("arg") {
        return result;
    }
    if let Ok(nested) = attr.parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated) {
        result.extend(nested.into_iter());
    }
    result
}

fn find_meta_value(metas: &[Meta], key: &str) -> Option<String> {
    for m in metas {
        if let Meta::NameValue(nv) = m {
            if nv.path.is_ident(key) {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        return Some(s.value());
                    }
                }
            }
        }
    }
    None
}

fn extract_struct_attrs(input: &DeriveInput) -> (Option<String>, Option<String>) {
    let metas: Vec<Meta> = input.attrs.iter().flat_map(parse_tui_attrs).collect();
    (find_meta_value(&metas, "title"), find_meta_value(&metas, "description"))
}

fn extract_label(field: &syn::Field) -> Option<String> {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    find_meta_value(&metas, "label")
}

fn extract_doc_comment(field: &syn::Field) -> Option<String> {
    let docs: Vec<String> = field.attrs.iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| {
            if let Meta::NameValue(nv) = &a.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        let t = s.value().trim().to_string();
                        if t.is_empty() { None } else { Some(t) }
                    } else { None }
                } else { None }
            } else { None }
        }).collect();
    if docs.is_empty() { None } else { Some(docs.join(" ")) }
}

fn determine_widget(field: &syn::Field, ty_str: &str) -> (proc_macro2::TokenStream, bool) {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    if let Some(w) = find_meta_value(&metas, "widget") {
        let widget = widget_from_str(&w);
        let is_opt = ty_str.starts_with("Option <");
        return (widget, is_opt);
    }
    let is_opt = ty_str.starts_with("Option <");
    let inner = if is_opt { extract_option_inner(ty_str) } else { ty_str.to_string() };
    (widget_for_type(&inner), is_opt)
}

fn widget_from_str(s: &str) -> proc_macro2::TokenStream {
    match s {
        "text" | "TextInput" => quote! { ::tui_generator::core::widget::WidgetKind::TextInput },
        "password" | "PasswordInput" => quote! { ::tui_generator::core::widget::WidgetKind::PasswordInput },
        "number" | "NumberInput" => quote! { ::tui_generator::core::widget::WidgetKind::NumberInput },
        "checkbox" | "Checkbox" => quote! { ::tui_generator::core::widget::WidgetKind::Checkbox },
        "select" | "Select" => quote! { ::tui_generator::core::widget::WidgetKind::Select },
        "multiselect" | "MultiSelect" => quote! { ::tui_generator::core::widget::WidgetKind::MultiSelect },
        "path" | "PathInput" => quote! { ::tui_generator::core::widget::WidgetKind::PathInput },
        "file" | "FileInput" => quote! { ::tui_generator::core::widget::WidgetKind::FileInput },
        "directory" | "DirectoryInput" => quote! { ::tui_generator::core::widget::WidgetKind::DirectoryInput },
        "confirm" | "Confirm" => quote! { ::tui_generator::core::widget::WidgetKind::Confirm },
        "textarea" | "TextArea" => quote! { ::tui_generator::core::widget::WidgetKind::TextArea },
        _ => quote! { ::tui_generator::core::widget::WidgetKind::TextInput },
    }
}

fn widget_for_type(ty: &str) -> proc_macro2::TokenStream {
    match ty {
        "bool" => quote! { ::tui_generator::core::widget::WidgetKind::Checkbox },
        "u8"|"u16"|"u32"|"u64"|"usize"|"i8"|"i16"|"i32"|"i64"|"f32"|"f64" =>
            quote! { ::tui_generator::core::widget::WidgetKind::NumberInput },
        s if s.contains("PathBuf") || s == "Path" || s.ends_with("::path::PathBuf") || s.ends_with("::path::Path") =>
            quote! { ::tui_generator::core::widget::WidgetKind::PathInput },
        _ => quote! { ::tui_generator::core::widget::WidgetKind::TextInput },
    }
}

fn extract_option_inner(ty_str: &str) -> String {
    let s = ty_str.trim();
    if s.starts_with("Option <") && s.ends_with('>') {
        s[8..s.len()-1].trim().to_string()
    } else {
        s.to_string()
    }
}

fn extract_default(field: &syn::Field, ty_str: &str, is_option: bool) -> (proc_macro2::TokenStream, bool) {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    for key in &["default", "default_value_t", "default_value"] {
        if let Some(val) = find_meta_value(&metas, key) {
            return (make_value_ts(&val), false);
        }
    }
    if is_option {
        (quote! { None }, false)
    } else {
        match type_default(ty_str) {
            Some(d) => (quote! { Some(#d) }, false),
            None => (quote! { None }, true),
        }
    }
}

fn make_value_ts(val: &str) -> proc_macro2::TokenStream {
    if let Ok(n) = val.parse::<i64>() {
        return quote! { Some(::tui_generator::core::value::Value::Integer(#n)) };
    }
    if let Ok(f) = val.parse::<f64>() {
        return quote! { Some(::tui_generator::core::value::Value::Float(#f)) };
    }
    if val == "true" { return quote! { Some(::tui_generator::core::value::Value::Bool(true)) }; }
    if val == "false" { return quote! { Some(::tui_generator::core::value::Value::Bool(false)) }; }
    quote! { Some(::tui_generator::core::value::Value::String(#val.to_string())) }
}

fn type_default(ty: &str) -> Option<proc_macro2::TokenStream> {
    match ty {
        "bool" => Some(quote! { ::tui_generator::core::value::Value::Bool(false) }),
        "u8"|"u16"|"u32"|"u64"|"usize"|"i8"|"i16"|"i32"|"i64" =>
            Some(quote! { ::tui_generator::core::value::Value::Integer(0) }),
        "f32"|"f64" => Some(quote! { ::tui_generator::core::value::Value::Float(0.0) }),
        "String"|"str" => Some(quote! { ::tui_generator::core::value::Value::String(String::new()) }),
        s if s.contains("PathBuf") || s == "Path" =>
            Some(quote! { ::tui_generator::core::value::Value::Path(std::path::PathBuf::new()) }),
        _ => None,
    }
}

fn extract_skip(field: &syn::Field) -> bool {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    metas.iter().any(|m| matches!(m, syn::Meta::Path(p) if p.is_ident("skip")))
}

fn extract_options(field: &syn::Field) -> Vec<String> {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    find_meta_value(&metas, "options")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

// --- Code generation helpers ---

fn gen_from_arm(name: &syn::Ident, ty_str: &str, is_option: bool, skip: bool) -> proc_macro2::TokenStream {
    let ns = name.to_string();
    if skip {
        return quote! { #name: Default::default() };
    }
    if is_option {
        let inner = extract_option_inner(ty_str);
        let conv = make_converter(&inner, true);
        return quote! {
            #name: match values.get(#ns) {
                Some(v) => #conv,
                None => None,
            }
        };
    }
    let conv = make_converter(ty_str, false);
    quote! {
        #name: match values.get(#ns) {
            Some(v) => #conv?,
            None => return Err(::tui_generator::core::error::TuiError::ConversionError(
                format!("missing field: {}", #ns)
            )),
        }
    }
}

fn make_converter(ty: &str, wrap_option: bool) -> proc_macro2::TokenStream {
    let err_msg = format!("expected {}", ty);

    let inner = match ty {
        "String"|"str" => quote! {
            match v {
                ::tui_generator::core::value::Value::String(s) => Ok(s.clone()),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "bool" => quote! {
            match v {
                ::tui_generator::core::value::Value::Bool(b) => Ok(*b),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "u8" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u8),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "u16" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u16),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "u32" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u32),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "u64" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u64),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "usize" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as usize),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "i8" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as i8),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "i16" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as i16),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "i32" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as i32),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "i64" => quote! {
            match v {
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "f32" => quote! {
            match v {
                ::tui_generator::core::value::Value::Float(f) => Ok(*f as f32),
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as f32),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        "f64" => quote! {
            match v {
                ::tui_generator::core::value::Value::Float(f) => Ok(*f),
                ::tui_generator::core::value::Value::Integer(n) => Ok(*n as f64),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        s if s.contains("PathBuf") || s == "Path" => quote! {
            match v {
                ::tui_generator::core::value::Value::Path(p) => Ok(p.clone()),
                ::tui_generator::core::value::Value::String(s) => Ok(std::path::PathBuf::from(s)),
                _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
            }
        },
        _ => quote! {
            Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into()))
        },
    };

    if wrap_option {
        quote! { #inner.ok() }
    } else {
        inner
    }
}

fn gen_to_arm(name: &syn::Ident, ty_str: &str) -> proc_macro2::TokenStream {
    let ns = name.to_string();
    let val_expr = match ty_str {
        "String"|"str" => quote! { ::tui_generator::core::value::Value::String(self.#name.clone()) },
        "bool" => quote! { ::tui_generator::core::value::Value::Bool(self.#name) },
        "u8"|"u16"|"u32"|"u64"|"usize"|"i8"|"i16"|"i32"|"i64" =>
            quote! { ::tui_generator::core::value::Value::Integer(self.#name as i64) },
        "f32"|"f64" =>
            quote! { ::tui_generator::core::value::Value::Float(self.#name as f64) },
        s if s.contains("PathBuf") || s == "Path" =>
            quote! { ::tui_generator::core::value::Value::Path(self.#name.clone()) },
        _ => quote! { ::tui_generator::core::value::Value::String(self.#name.to_string()) },
    };
    quote! { map.insert(#ns.to_string(), #val_expr); }
}

// --- Utility functions ---

fn type_to_string(ty: &syn::Type) -> String {
    quote!(#ty).to_string()
}

fn to_title_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
