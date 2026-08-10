use crate::value::Value;
use crate::widget::WidgetKind;
use crate::validation::Constraint;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuiSchema {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub subcommands: Vec<Command>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<Value>,
    pub widget: WidgetKind,
    pub constraints: Vec<Constraint>,
    pub options: Vec<String>,
    pub skip: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Command {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub subcommands: Vec<Command>,
}
