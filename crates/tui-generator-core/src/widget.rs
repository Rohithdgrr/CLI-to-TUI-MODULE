#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WidgetKind {
    TextInput,
    PasswordInput,
    NumberInput,
    Checkbox,
    Select,
    MultiSelect,
    PathInput,
    FileInput,
    DirectoryInput,
    Confirm,
    TextArea,
}

impl WidgetKind {
    pub fn label(&self) -> &'static str {
        match self {
            WidgetKind::TextInput => "Text Input",
            WidgetKind::PasswordInput => "Password Input",
            WidgetKind::NumberInput => "Number Input",
            WidgetKind::Checkbox => "Checkbox",
            WidgetKind::Select => "Select",
            WidgetKind::MultiSelect => "Multi Select",
            WidgetKind::PathInput => "Path Input",
            WidgetKind::FileInput => "File Input",
            WidgetKind::DirectoryInput => "Directory Input",
            WidgetKind::Confirm => "Confirm",
            WidgetKind::TextArea => "Text Area",
        }
    }
}
