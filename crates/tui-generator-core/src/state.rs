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
        }
    }

    pub fn set_value(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn focus_next(&mut self, total_fields: usize) {
        self.focused_index = (self.focused_index + 1) % total_fields;
        self.editing = false;
        self.cursor_pos = 0;
        self.select_index = 0;
    }

    pub fn focus_prev(&mut self, total_fields: usize) {
        self.focused_index = if self.focused_index == 0 {
            total_fields - 1
        } else {
            self.focused_index - 1
        };
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
}
