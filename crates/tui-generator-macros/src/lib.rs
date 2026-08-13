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

        let label = extract_label(field).unwrap_or_else(|| to_title_case(&field_name_str));
        let doc = extract_doc_comment(field);
        let doc_ts = match &doc {
            Some(d) => quote! { Some(#d.to_string()) },
            None => quote! { None },
        };
        let (widget_ts, is_option) = determine_widget(field, &ty);
        let (default_ts, required) = extract_default(field, &ty, is_option);
        let options = extract_options(field);
        let skip = extract_skip(field);
        let section = extract_section(field);
        let readonly = extract_readonly(field);

        let options_tokens: Vec<proc_macro2::TokenStream> = options.iter()
            .map(|o| quote! { #o.to_string() })
            .collect();

        let section_ts = match &section {
            Some(s) => quote! { Some(#s.to_string()) },
            None => quote! { None },
        };

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
                section: #section_ts,
                readonly: #readonly,
            }
        });

        from_value_arms.push(gen_from_arm(field_name, ty, is_option, skip));
        if !skip {
            to_value_arms.push(gen_to_arm(field_name, &ty));
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
                // Fix close bug: open TUI by default unless CLI args explicitly provided
                // This prevents 1-second exit when running in a directory
                if args.is_empty() || args.iter().any(|a| a == "--tui") {
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

fn determine_widget(field: &syn::Field, ty: &syn::Type) -> (proc_macro2::TokenStream, bool) {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    let is_opt = is_option_type(ty);
    if let Some(w) = find_meta_value(&metas, "widget") {
        let widget = widget_from_str(&w);
        return (widget, is_opt);
    }
    if !extract_options(field).is_empty() {
        return (
            quote! { ::tui_generator::core::widget::WidgetKind::Select },
            is_opt,
        );
    }
    type_to_widget_kind(ty)
}

fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            result.push(' ');
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn is_option_type(ty: &syn::Type) -> bool {
    last_type_ident(ty).map_or(false, |i| i == "Option")
}

fn is_vec_type(ty: &syn::Type) -> bool {
    last_type_ident(ty).map_or(false, |i| i == "Vec")
}

fn last_type_ident(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(type_path) = ty {
        type_path.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

fn first_generic_arg(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    return Some(inner);
                }
            }
        }
    }
    None
}

fn extract_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if is_option_type(ty) {
        first_generic_arg(ty)
    } else {
        None
    }
}

fn type_to_widget_kind(ty: &syn::Type) -> (proc_macro2::TokenStream, bool) {
    let is_opt = is_option_type(ty);
    let mut effective: &syn::Type = ty;
    if is_opt {
        effective = extract_option_inner_type(ty).unwrap_or(ty);
    }
    if is_vec_type(effective) {
        return (
            quote! { ::tui_generator::core::widget::WidgetKind::MultiSelect },
            is_opt,
        );
    }
    let widget = match last_type_ident(effective).as_deref() {
        Some("bool") => quote! { ::tui_generator::core::widget::WidgetKind::Checkbox },
        Some("PathBuf") | Some("Path") => {
            quote! { ::tui_generator::core::widget::WidgetKind::PathInput }
        }
        Some("u8") | Some("u16") | Some("u32") | Some("u64") | Some("usize")
        | Some("i8") | Some("i16") | Some("i32") | Some("i64")
        | Some("f32") | Some("f64") => {
            quote! { ::tui_generator::core::widget::WidgetKind::NumberInput }
        }
        _ => quote! { ::tui_generator::core::widget::WidgetKind::TextInput },
    };
    (widget, is_opt)
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

fn extract_default(field: &syn::Field, ty: &syn::Type, is_option: bool) -> (proc_macro2::TokenStream, bool) {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    for key in &["default", "default_value_t", "default_value"] {
        if let Some(val) = find_meta_value(&metas, key) {
            return (make_value_ts(&val), false);
        }
    }
    if is_option {
        (quote! { None }, false)
    } else {
        match type_default_from_type(ty) {
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

fn type_default_from_type(ty: &syn::Type) -> Option<proc_macro2::TokenStream> {
    if let syn::Type::Path(type_path) = ty {
        let ident = type_path.path.segments.last().map(|s| s.ident.to_string());
        match ident.as_deref() {
            Some("bool") => Some(quote! { ::tui_generator::core::value::Value::Bool(false) }),
            Some("u8") | Some("u16") | Some("u32") | Some("u64") | Some("usize")
            | Some("i8") | Some("i16") | Some("i32") | Some("i64") => {
                Some(quote! { ::tui_generator::core::value::Value::Integer(0) })
            }
            Some("f32") | Some("f64") => Some(quote! { ::tui_generator::core::value::Value::Float(0.0) }),
            Some("String") | Some("str") => Some(quote! { ::tui_generator::core::value::Value::String(String::new()) }),
            Some("PathBuf") | Some("Path") => {
                Some(quote! { ::tui_generator::core::value::Value::Path(std::path::PathBuf::new()) })
            }
            Some("Vec") => {
                Some(quote! { ::tui_generator::core::value::Value::List(vec![]) })
            }
            _ => None,
        }
    } else {
        None
    }
}

fn extract_skip(field: &syn::Field) -> bool {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    metas.iter().any(|m| matches!(m, syn::Meta::Path(p) if p.is_ident("skip")))
}

fn extract_readonly(field: &syn::Field) -> bool {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    metas.iter().any(|m| matches!(m, syn::Meta::Path(p) if p.is_ident("readonly")))
}

fn extract_section(field: &syn::Field) -> Option<String> {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    find_meta_value(&metas, "section")
}

fn extract_options(field: &syn::Field) -> Vec<String> {
    let metas: Vec<Meta> = field.attrs.iter().flat_map(parse_tui_attrs).collect();
    find_meta_value(&metas, "options")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

// --- Code generation helpers ---

fn gen_from_arm(name: &syn::Ident, ty: &syn::Type, is_option: bool, skip: bool) -> proc_macro2::TokenStream {
    let ns = name.to_string();
    if skip {
        return quote! { #name: Default::default() };
    }
    if is_option {
        let inner = extract_option_inner_type(ty).unwrap_or(ty);
        let conv = make_converter_from_type(&inner, true);
        return quote! {
            #name: match values.get(#ns) {
                Some(v) => #conv,
                None => None,
            }
        };
    }
    let conv = make_converter_from_type(ty, false);
    quote! {
        #name: match values.get(#ns) {
            Some(v) => #conv?,
            None => return Err(::tui_generator::core::error::TuiError::ConversionError(
                format!("missing field: {}", #ns)
            )),
        }
    }
}

fn make_converter_from_type(ty: &syn::Type, wrap_option: bool) -> proc_macro2::TokenStream {
    let err_msg = format!("expected proper type");

    let inner = match ty {
        syn::Type::Path(type_path) => {
            let ident = type_path.path.segments.last().map(|s| s.ident.to_string());
            match ident.as_deref() {
                Some("String") | Some("str") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::String(s) => Ok(s.clone()),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("bool") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Bool(b) => Ok(*b),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("u8") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u8),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("u16") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u16),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("u32") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u32),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("u64") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as u64),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("usize") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as usize),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("i8") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as i8),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("i16") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as i16),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("i32") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as i32),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("i64") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("f32") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Float(f) => Ok(*f as f32),
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as f32),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("f64") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Float(f) => Ok(*f),
                        ::tui_generator::core::value::Value::Integer(n) => Ok(*n as f64),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("PathBuf") | Some("Path") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::Path(p) => Ok(p.clone()),
                        ::tui_generator::core::value::Value::String(s) => Ok(std::path::PathBuf::from(s)),
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                Some("Vec") => quote! {
                    match v {
                        ::tui_generator::core::value::Value::List(items) => {
                            Ok(items.iter().filter_map(|item| {
                                match item {
                                    ::tui_generator::core::value::Value::String(s) => Some(s.clone()),
                                    _ => None,
                                }
                            }).collect())
                        }
                        _ => Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into())),
                    }
                },
                _ => quote! {
                    Err(::tui_generator::core::error::TuiError::ConversionError(#err_msg.into()))
                },
            }
        }
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

fn gen_to_arm(name: &syn::Ident, ty: &syn::Type) -> proc_macro2::TokenStream {
    let ns = name.to_string();
    let val_expr = match ty {
        syn::Type::Path(type_path) => {
            let ident = type_path.path.segments.last().map(|s| s.ident.to_string());
            match ident.as_deref() {
                Some("String") | Some("str") => {
                    quote! { ::tui_generator::core::value::Value::String(self.#name.clone()) }
                }
                Some("bool") => quote! { ::tui_generator::core::value::Value::Bool(self.#name) },
                Some("u8") | Some("u16") | Some("u32") | Some("u64") | Some("usize")
                | Some("i8") | Some("i16") | Some("i32") | Some("i64") => {
                    quote! { ::tui_generator::core::value::Value::Integer(self.#name as i64) }
                }
                Some("f32") | Some("f64") => {
                    quote! { ::tui_generator::core::value::Value::Float(self.#name as f64) }
                }
                Some("PathBuf") | Some("Path") => {
                    quote! { ::tui_generator::core::value::Value::Path(self.#name.clone()) }
                }
                Some("Vec") => {
                    quote! {
                        ::tui_generator::core::value::Value::List(
                            self.#name.iter().map(|s| ::tui_generator::core::value::Value::String(s.clone())).collect()
                        )
                    }
                }
                _ => quote! { ::tui_generator::core::value::Value::String(self.#name.to_string()) },
            }
        }
        _ => quote! { ::tui_generator::core::value::Value::String(self.#name.to_string()) },
    };
    quote! { map.insert(#ns.to_string(), #val_expr); }
}
