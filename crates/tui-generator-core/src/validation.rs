#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Constraint {
    Required,
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String),
    Custom(String),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub constraint: Option<Constraint>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate(schema: &[super::schema::Field], values: &std::collections::HashMap<String, super::value::Value>) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for field in schema {
        let value = values.get(&field.name);

        let is_missing = value.is_none()
            || matches!(value, Some(super::value::Value::None))
            || matches!(value, Some(super::value::Value::String(s)) if s.is_empty());

        if field.required && is_missing {
            let already = field.constraints.iter().any(|c| matches!(c, Constraint::Required));
            if !already {
                errors.push(ValidationError {
                    field: field.name.clone(),
                    message: format!("{} is required", field.label),
                    constraint: Some(Constraint::Required),
                });
            }
        }

        for constraint in &field.constraints {
            match constraint {
                Constraint::Required => {
                    if is_missing {
                        errors.push(ValidationError {
                            field: field.name.clone(),
                            message: format!("{} is required", field.label),
                            constraint: Some(constraint.clone()),
                        });
                    }
                }
                Constraint::MinLength(min) => {
                    if let Some(super::value::Value::String(s)) = value {
                        if s.len() < *min {
                            errors.push(ValidationError {
                                field: field.name.clone(),
                                message: format!("{} must be at least {} characters", field.label, min),
                                constraint: Some(constraint.clone()),
                            });
                        }
                    }
                }
                Constraint::MaxLength(max) => {
                    if let Some(super::value::Value::String(s)) = value {
                        if s.len() > *max {
                            errors.push(ValidationError {
                                field: field.name.clone(),
                                message: format!("{} must be at most {} characters", field.label, max),
                                constraint: Some(constraint.clone()),
                            });
                        }
                    }
                }
                Constraint::MinValue(min) => {
                    if let Some(v) = value {
                        let num = match v {
                            super::value::Value::Integer(i) => *i as f64,
                            super::value::Value::Float(f) => *f,
                            _ => continue,
                        };
                        if num < *min {
                            errors.push(ValidationError {
                                field: field.name.clone(),
                                message: format!("{} must be at least {}", field.label, min),
                                constraint: Some(constraint.clone()),
                            });
                        }
                    }
                }
                Constraint::MaxValue(max) => {
                    if let Some(v) = value {
                        let num = match v {
                            super::value::Value::Integer(i) => *i as f64,
                            super::value::Value::Float(f) => *f,
                            _ => continue,
                        };
                        if num > *max {
                            errors.push(ValidationError {
                                field: field.name.clone(),
                                message: format!("{} must be at most {}", field.label, max),
                                constraint: Some(constraint.clone()),
                            });
                        }
                    }
                }
                Constraint::Pattern(_) | Constraint::Custom(_) => {}
            }
        }
    }

    errors
}
