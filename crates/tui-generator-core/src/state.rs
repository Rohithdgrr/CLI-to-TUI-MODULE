use std::collections::HashMap;
use crate::schema::TuiSchema;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct FormState {
    pub values: HashMap<String, Value>,
    pub focused_index: usize,
    pub scroll_offset: usize,
    pub editing: bool,
    pub cursor_pos: usize,
    pub select_index: usize,
    pub errors: Vec<crate::validation::ValidationError>,
    pub help_visible: bool,
}

impl FormState {
    pub fn from_schema(schema: &TuiSchema) -> Self {
        let mut values = HashMap::new();
        for field in &schema.fields {
            if let Some(default) = &field.default {
                values.insert(field.name.clone(), default.clone());
            } else {
                values.insert(field.name.clone(), Value::None);
            }
        }
        Self {
            values,
            focused_index: 0,
            scroll_offset: 0,
            editing: false,
            cursor_pos: 0,
            select_index: 0,
            errors: Vec::new(),
            help_visible: false,
        }
    }

    pub fn set_value(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn focus_next(&mut self, schema: &TuiSchema) {
        let total = schema.fields.len();
        if total == 0 { return; }
        let start = self.focused_index;
        let mut idx = (start + 1) % total;
        while idx != start && schema.fields[idx].skip {
            idx = (idx + 1) % total;
        }
        self.focused_index = idx;
        self.editing = false;
        self.cursor_pos = 0;
        self.select_index = 0;
    }

    pub fn focus_prev(&mut self, schema: &TuiSchema) {
        let total = schema.fields.len();
        if total == 0 { return; }
        let start = self.focused_index;
        let mut idx = if start == 0 { total - 1 } else { start - 1 };
        while idx != start && schema.fields[idx].skip {
            idx = if idx == 0 { total - 1 } else { idx - 1 };
        }
        self.focused_index = idx;
        self.editing = false;
        self.cursor_pos = 0;
        self.select_index = 0;
    }

    pub fn validate(&mut self, schema: &TuiSchema) {
        self.errors = crate::validation::validate(&schema.fields, &self.values);
    }

    pub fn edit_buffer(&self, field_name: &str) -> String {
        match self.values.get(field_name) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Integer(n)) => n.to_string(),
            Some(Value::Float(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Path(p)) => p.to_string_lossy().to_string(),
            _ => String::new(),
        }
    }

    pub fn set_edit_buffer(&mut self, field_name: &str, buffer: String, widget: crate::widget::WidgetKind) {
        let value = match widget {
            crate::widget::WidgetKind::NumberInput => {
                if let Ok(n) = buffer.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = buffer.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::String(buffer)
                }
            }
            crate::widget::WidgetKind::Checkbox => Value::Bool(buffer == "true"),
            _ => Value::String(buffer),
        };
        self.values.insert(field_name.to_string(), value);
    }

    #[cfg(feature = "serde")]
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), crate::error::TuiError> {
        let json = serde_json::to_string_pretty(&self.values)
            .map_err(|e| crate::error::TuiError::ConversionError(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    #[cfg(feature = "serde")]
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), crate::error::TuiError> {
        let json = std::fs::read_to_string(path)?;
        let values: HashMap<String, Value> = serde_json::from_str(&json)
            .map_err(|e| crate::error::TuiError::ConversionError(e.to_string()))?;
        for (key, value) in values {
            self.values.insert(key, value);
        }
        Ok(())
    }

    pub fn to_cli_args(&self, schema: &TuiSchema) -> Vec<String> {
        let mut args = Vec::new();
        for field in &schema.fields {
            if field.skip {
                continue;
            }
            if let Some(val) = self.values.get(&field.name) {
                match val {
                    Value::String(s) => {
                        args.push(format!("--{}", field.name));
                        args.push(s.clone());
                    }
                    Value::Integer(n) => {
                        args.push(format!("--{}", field.name));
                        args.push(n.to_string());
                    }
                    Value::Float(f) => {
                        args.push(format!("--{}", field.name));
                        args.push(f.to_string());
                    }
                    Value::Bool(true) => {
                        args.push(format!("--{}", field.name));
                    }
                    Value::Path(p) => {
                        args.push(format!("--{}", field.name));
                        args.push(p.to_string_lossy().to_string());
                    }
                    Value::List(items) => {
                        for item in items {
                            match item {
                                Value::String(s) => {
                                    args.push(format!("--{}", field.name));
                                    args.push(s.clone());
                                }
                                Value::Integer(n) => {
                                    args.push(format!("--{}", field.name));
                                    args.push(n.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                    Value::Bool(false) | Value::None => {}
                }
            }
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{TuiSchema, Field};
    use crate::widget::WidgetKind;

    fn test_schema_with_skips(skips: Vec<bool>) -> TuiSchema {
        TuiSchema {
            name: "Test".into(),
            description: None,
            fields: skips.iter().enumerate().map(|(i, &skip)| Field {
                name: format!("f{}", i),
                label: format!("Field {}", i),
                description: None,
                required: false,
                default: None,
                widget: WidgetKind::TextInput,
                constraints: vec![],
                options: vec![],
                skip,
                section: None,
                readonly: false,
            }).collect(),
            subcommands: vec![],
        }
    }

    #[test]
    fn test_focus_next_skips_hidden() {
        let schema = test_schema_with_skips(vec![false, true, false]);
        let mut state = FormState::from_schema(&schema);
        assert_eq!(state.focused_index, 0);
        state.focus_next(&schema);
        assert_eq!(state.focused_index, 2);
    }

    #[test]
    fn test_focus_prev_skips_hidden() {
        let schema = test_schema_with_skips(vec![false, true, false]);
        let mut state = FormState::from_schema(&schema);
        state.focused_index = 2;
        state.focus_prev(&schema);
        assert_eq!(state.focused_index, 0);
    }

    #[test]
    fn test_focus_next_all_visible() {
        let schema = test_schema_with_skips(vec![false, false, false]);
        let mut state = FormState::from_schema(&schema);
        state.focus_next(&schema);
        assert_eq!(state.focused_index, 1);
        state.focus_next(&schema);
        assert_eq!(state.focused_index, 2);
    }

    #[test]
    fn test_set_edit_buffer_number() {
        let schema = test_schema_with_skips(vec![false]);
        let mut state = FormState::from_schema(&schema);
        state.set_edit_buffer("f0", "42".into(), WidgetKind::NumberInput);
        assert_eq!(state.get_value("f0"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_set_edit_buffer_checkbox() {
        let schema = test_schema_with_skips(vec![false]);
        let mut state = FormState::from_schema(&schema);
        state.set_edit_buffer("f0", "true".into(), WidgetKind::Checkbox);
        assert_eq!(state.get_value("f0"), Some(&Value::Bool(true)));
    }
}
