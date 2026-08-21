use std::path::PathBuf;
use tui_generator_core::schema::{TuiSchema, Field};
use tui_generator_core::widget::WidgetKind;
use tui_generator_core::validation::Constraint;
use tui_generator_ratatui::RatatuiRenderer;

fn main() {
    let schema = TuiSchema {
        name: "Image Processor".into(),
        description: Some("Convert any webpage into a desktop app".into()),
        fields: vec![
            Field {
                name: "input".into(),
                label: "Input Path".into(),
                description: Some("Source image to process".into()),
                required: true,
                default: Some(tui_generator_core::value::Value::String("./input.png".into())),
                widget: WidgetKind::PathInput,
                constraints: vec![Constraint::Required],
                options: vec![],
                skip: false,
                section: Some("Source".into()),
                readonly: false,
            },
            Field {
                name: "output".into(),
                label: "Output Path".into(),
                description: Some("Destination for processed image".into()),
                required: true,
                default: Some(tui_generator_core::value::Value::String("./output.jpg".into())),
                widget: WidgetKind::PathInput,
                constraints: vec![Constraint::Required],
                options: vec![],
                skip: false,
                section: Some("Source".into()),
                readonly: false,
            },
            Field {
                name: "width".into(),
                label: "Window Width".into(),
                description: None,
                required: false,
                default: Some(tui_generator_core::value::Value::Integer(1200)),
                widget: WidgetKind::NumberInput,
                constraints: vec![],
                options: vec![],
                skip: false,
                section: Some("Window".into()),
                readonly: false,
            },
            Field {
                name: "height".into(),
                label: "Window Height".into(),
                description: None,
                required: false,
                default: Some(tui_generator_core::value::Value::Integer(800)),
                widget: WidgetKind::NumberInput,
                constraints: vec![],
                options: vec![],
                skip: false,
                section: Some("Window".into()),
                readonly: false,
            },
            Field {
                name: "fullscreen".into(),
                label: "Start Fullscreen".into(),
                description: None,
                required: false,
                default: Some(tui_generator_core::value::Value::Bool(false)),
                widget: WidgetKind::Checkbox,
                constraints: vec![],
                options: vec![],
                skip: false,
                section: Some("Window".into()),
                readonly: false,
            },
            Field {
                name: "format".into(),
                label: "Output Format".into(),
                description: None,
                required: false,
                default: Some(tui_generator_core::value::Value::String("png".into())),
                widget: WidgetKind::Select,
                constraints: vec![],
                options: vec!["png".into(), "jpg".into(), "webp".into(), "gif".into()],
                skip: false,
                section: Some("Options".into()),
                readonly: false,
            },
            Field {
                name: "verbose".into(),
                label: "Verbose Output".into(),
                description: None,
                required: false,
                default: Some(tui_generator_core::value::Value::Bool(false)),
                widget: WidgetKind::Checkbox,
                constraints: vec![],
                options: vec![],
                skip: false,
                section: Some("Options".into()),
                readonly: false,
            },
        ],
        subcommands: vec![],
    };

    println!("Starting TUI demo... Press any key in the TUI to continue after splash.");
    
    match RatatuiRenderer::new().run_tui(&schema) {
        Ok(state) => {
            println!("\n✓ Form submitted successfully!");
            println!("Values: {:?}", state.values);
        }
        Err(e) => {
            eprintln!("\n✗ TUI error or cancelled: {}", e);
        }
    }
}