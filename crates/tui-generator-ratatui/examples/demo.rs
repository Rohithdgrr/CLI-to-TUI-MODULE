use tui_generator_core::schema::{Field, TuiSchema};
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;
use tui_generator_core::validation::Constraint;
use tui_generator_ratatui::RatatuiRenderer;

fn build_schema() -> TuiSchema {
    TuiSchema {
        name: "Image Processor".to_string(),
        description: Some("Configure image processing options".to_string()),
        fields: vec![
            Field {
                name: "input".to_string(),
                label: "Input File".to_string(),
                description: Some("Path to input image".to_string()),
                required: true,
                default: Some(Value::String("./input.png".to_string())),
                widget: WidgetKind::TextInput,
                constraints: vec![],
                options: vec![],
                skip: false,
            },
            Field {
                name: "output".to_string(),
                label: "Output File".to_string(),
                description: Some("Path to output image".to_string()),
                required: true,
                default: Some(Value::String("./output.png".to_string())),
                widget: WidgetKind::TextInput,
                constraints: vec![],
                options: vec![],
                skip: false,
            },
            Field {
                name: "threads".to_string(),
                label: "Threads".to_string(),
                description: Some("Number of processing threads".to_string()),
                required: false,
                default: Some(Value::Integer(4)),
                widget: WidgetKind::NumberInput,
                constraints: vec![Constraint::MinValue(1.0), Constraint::MaxValue(64.0)],
                options: vec![],
                skip: false,
            },
            Field {
                name: "format".to_string(),
                label: "Output Format".to_string(),
                description: Some("Image output format".to_string()),
                required: false,
                default: Some(Value::String("png".to_string())),
                widget: WidgetKind::Select,
                constraints: vec![],
                options: vec![
                    "png".to_string(),
                    "jpg".to_string(),
                    "webp".to_string(),
                    "gif".to_string(),
                ],
                skip: false,
            },
            Field {
                name: "quality".to_string(),
                label: "Quality".to_string(),
                description: Some("Compression quality (1-100)".to_string()),
                required: false,
                default: Some(Value::Integer(85)),
                widget: WidgetKind::NumberInput,
                constraints: vec![Constraint::MinValue(1.0), Constraint::MaxValue(100.0)],
                options: vec![],
                skip: false,
            },
            Field {
                name: "verbose".to_string(),
                label: "Verbose".to_string(),
                description: Some("Enable verbose output".to_string()),
                required: false,
                default: Some(Value::Bool(false)),
                widget: WidgetKind::Checkbox,
                constraints: vec![],
                options: vec![],
                skip: false,
            },
            Field {
                name: "overwrite".to_string(),
                label: "Overwrite".to_string(),
                description: Some("Overwrite existing files".to_string()),
                required: false,
                default: Some(Value::Bool(true)),
                widget: WidgetKind::Checkbox,
                constraints: vec![],
                options: vec![],
                skip: false,
            },
        ],
        subcommands: vec![],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = build_schema();

    println!("Launching TUI...");
    println!("Use arrow keys to navigate, Enter to edit, Space to toggle checkboxes");
    println!("Press 'q' or Esc to quit.\n");

    match RatatuiRenderer::run(&schema) {
        Ok(state) => {
            println!("\n--- Submitted Values ---");
            for field in &schema.fields {
                match state.get_value(&field.name) {
                    Some(Value::String(s)) => println!("  {}: {}", field.label, s),
                    Some(Value::Integer(n)) => println!("  {}: {}", field.label, n),
                    Some(Value::Float(n)) => println!("  {}: {}", field.label, n),
                    Some(Value::Bool(b)) => println!("  {}: {}", field.label, b),
                    _ => println!("  {}: (empty)", field.label),
                }
            }
        }
        Err(e) => {
            eprintln!("TUI exited: {}", e);
        }
    }

    Ok(())
}
