use tui_generator_core::schema::TuiSchema;
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;

pub trait ClapAdapter {
    fn to_schema() -> TuiSchema;
}

pub fn type_to_widget(ty: &str) -> WidgetKind {
    match ty {
        "String" | "str" => WidgetKind::TextInput,
        "PathBuf" | "Path" => WidgetKind::PathInput,
        "bool" => WidgetKind::Checkbox,
        "u8" | "u16" | "u32" | "u64" | "usize"
        | "i8" | "i16" | "i32" | "i64" => WidgetKind::NumberInput,
        "f32" | "f64" => WidgetKind::NumberInput,
        _ => WidgetKind::TextInput,
    }
}

pub fn default_for_type(ty: &str) -> Option<Value> {
    match ty {
        "bool" => Some(Value::Bool(false)),
        "u8" | "u16" | "u32" | "u64" | "usize"
        | "i8" | "i16" | "i32" | "i64" => Some(Value::Integer(0)),
        "f32" | "f64" => Some(Value::Float(0.0)),
        "String" | "str" => Some(Value::String(String::new())),
        _ => None,
    }
}
