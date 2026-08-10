use std::collections::HashMap;
use crate::schema::TuiSchema;
use crate::value::Value;
use crate::error::TuiError;

pub trait Tui {
    fn schema() -> TuiSchema;
    fn from_values(values: &HashMap<String, Value>) -> Result<Self, TuiError>
    where
        Self: Sized;
    fn to_values(&self) -> HashMap<String, Value>;
}
