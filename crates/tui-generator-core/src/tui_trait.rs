use crate::schema::TuiSchema;
use crate::error::TuiError;

pub trait Tui: Sized {
    fn tui_schema() -> TuiSchema;
    fn parse_or_tui() -> Result<Self, TuiError>;
}