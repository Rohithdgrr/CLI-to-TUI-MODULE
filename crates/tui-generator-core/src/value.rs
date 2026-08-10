use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    String(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Path(PathBuf),
    List(Vec<Value>),
    None,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Path(_) => "path",
            Value::List(_) => "list",
            Value::None => "none",
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Integer(i as i64)
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Self {
        Value::Integer(u.min(i64::MAX as u64) as i64)
    }
}

impl From<u32> for Value {
    fn from(u: u32) -> Self {
        Value::Integer(u as i64)
    }
}

impl From<usize> for Value {
    fn from(u: usize) -> Self {
        Value::Integer(u.min(i64::MAX as usize) as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f as f64)
    }
}

impl From<PathBuf> for Value {
    fn from(p: PathBuf) -> Self {
        Value::Path(p)
    }
}
