pub mod schema;
pub mod value;
pub mod widget;
pub mod validation;
pub mod state;
pub mod event;
pub mod error;
pub mod tui_trait;

pub use tui_trait::Tui;

#[cfg(test)]
mod tests {
    use crate::value::Value;
    use crate::validation::{validate, Constraint, ValidationError};
    use crate::schema::Field;
    use crate::widget::WidgetKind;
    use std::collections::HashMap;

    fn make_field(name: &str, widget: WidgetKind, constraints: Vec<Constraint>) -> Field {
        Field {
            name: name.to_string(),
            label: name.to_string(),
            description: None,
            required: false,
            default: None,
            widget,
            constraints,
            options: vec![],
            skip: false,
            section: None,
        }
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(Value::String("x".into()).type_name(), "string");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Integer(0).type_name(), "integer");
        assert_eq!(Value::Float(0.0).type_name(), "float");
        assert_eq!(Value::None.type_name(), "none");
    }

    #[test]
    fn test_value_from_conversions() {
        let v: Value = "hello".into();
        assert_eq!(v, Value::String("hello".into()));

        let v: Value = String::from("world").into();
        assert_eq!(v, Value::String("world".into()));

        let v: Value = true.into();
        assert_eq!(v, Value::Bool(true));

        let v: Value = 42i64.into();
        assert_eq!(v, Value::Integer(42));

        let v: Value = 3.14f64.into();
        assert_eq!(v, Value::Float(3.14));
    }

    #[test]
    fn test_validate_required_missing() {
        let fields = vec![make_field("name", WidgetKind::TextInput, vec![Constraint::Required])];
        let values = HashMap::new();
        let errors = validate(&fields, &values);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn test_validate_required_present() {
        let fields = vec![make_field("name", WidgetKind::TextInput, vec![Constraint::Required])];
        let mut values = HashMap::new();
        values.insert("name".to_string(), Value::String("Alice".into()));
        let errors = validate(&fields, &values);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_min_length() {
        let fields = vec![make_field("name", WidgetKind::TextInput, vec![Constraint::MinLength(3)])];
        let mut values = HashMap::new();
        values.insert("name".to_string(), Value::String("ab".into()));
        let errors = validate(&fields, &values);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validate_max_length_ok() {
        let fields = vec![make_field("name", WidgetKind::TextInput, vec![Constraint::MaxLength(10)])];
        let mut values = HashMap::new();
        values.insert("name".to_string(), Value::String("short".into()));
        let errors = validate(&fields, &values);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_min_value() {
        let fields = vec![make_field("age", WidgetKind::NumberInput, vec![Constraint::MinValue(18.0)])];
        let mut values = HashMap::new();
        values.insert("age".to_string(), Value::Integer(15));
        let errors = validate(&fields, &values);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validate_max_value_ok() {
        let fields = vec![make_field("age", WidgetKind::NumberInput, vec![Constraint::MaxValue(100.0)])];
        let mut values = HashMap::new();
        values.insert("age".to_string(), Value::Integer(50));
        let errors = validate(&fields, &values);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError {
            field: "email".into(),
            message: "email is required".into(),
            constraint: None,
        };
        assert_eq!(format!("{}", err), "email: email is required");
    }

    #[test]
    fn test_field_with_section() {
        let field = make_field("host", WidgetKind::TextInput, vec![]);
        assert_eq!(field.section, None);

        let field_with_section = Field {
            name: "host".into(),
            label: "Host".into(),
            description: None,
            required: true,
            default: None,
            widget: WidgetKind::TextInput,
            constraints: vec![],
            options: vec![],
            skip: false,
            section: Some("Network".into()),
        };
        assert_eq!(field_with_section.section, Some("Network".into()));
    }

    #[test]
    fn test_skip_field_uses_default() {
        let field = Field {
            name: "internal_id".into(),
            label: "Internal ID".into(),
            description: None,
            required: false,
            default: Some(Value::Integer(0)),
            widget: WidgetKind::TextInput,
            constraints: vec![],
            options: vec![],
            skip: true,
            section: None,
        };
        assert!(field.skip);
    }
}
