use tui_generator_core::schema::{Field, TuiSchema};
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;
use tui_generator_core::validation::Constraint;

fn build_schema() -> TuiSchema {
    TuiSchema {
        name: "Image Processor".to_string(),
        description: Some("Configure image processing options".to_string()),
        fields: vec![
            Field {
                name: "input".to_string(),
                label: "Input File".to_string(),
                required: true,
                default: Some(Value::String("./input.png".to_string())),
                widget: WidgetKind::TextInput,
                constraints: vec![],
                options: vec![],
                description: Some("Path to input image".to_string()),
                skip: false,
            },
            Field {
                name: "output".to_string(),
                label: "Output File".to_string(),
                required: true,
                default: Some(Value::String("./output.png".to_string())),
                widget: WidgetKind::TextInput,
                constraints: vec![],
                options: vec![],
                description: Some("Path to output image".to_string()),
                skip: false,
            },
            Field {
                name: "threads".to_string(),
                label: "Threads".to_string(),
                required: false,
                default: Some(Value::Integer(4)),
                widget: WidgetKind::NumberInput,
                constraints: vec![Constraint::MinValue(1.0), Constraint::MaxValue(64.0)],
                options: vec![],
                description: Some("Number of processing threads".to_string()),
                skip: false,
            },
            Field {
                name: "format".to_string(),
                label: "Output Format".to_string(),
                required: false,
                default: Some(Value::String("png".to_string())),
                widget: WidgetKind::Select,
                constraints: vec![],
                options: vec!["png".into(), "jpg".into(), "webp".into(), "gif".into()],
                description: Some("Image output format".to_string()),
                skip: false,
            },
            Field {
                name: "quality".to_string(),
                label: "Quality".to_string(),
                required: false,
                default: Some(Value::Integer(85)),
                widget: WidgetKind::NumberInput,
                constraints: vec![Constraint::MinValue(1.0), Constraint::MaxValue(100.0)],
                options: vec![],
                description: Some("Compression quality (1-100)".to_string()),
                skip: false,
            },
            Field {
                name: "verbose".to_string(),
                label: "Verbose".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                widget: WidgetKind::Checkbox,
                constraints: vec![],
                options: vec![],
                description: Some("Enable verbose output".to_string()),
                skip: false,
            },
            Field {
                name: "overwrite".to_string(),
                label: "Overwrite".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                widget: WidgetKind::Checkbox,
                constraints: vec![],
                options: vec![],
                description: Some("Overwrite existing files".to_string()),
                skip: false,
            },
        ],
        subcommands: vec![],
    }
}

fn main() {
    let schema = build_schema();

    println!();
    println!("  === TUI PREVIEW (no terminal takeover) ===");
    println!();

    let border = "+".to_owned() + &"-".repeat(68) + "+";
    let empty = "|".to_owned() + &" ".repeat(68) + "|";

    let centered = |s: &str, w: usize| -> String {
        let pad = w.saturating_sub(s.len());
        let l = pad / 2;
        let r = pad - l;
        format!("{}{}{}", " ".repeat(l), s, " ".repeat(r))
    };

    // Header
    println!("{}", border);
    println!("|{}|", centered(&format!("{}  {}", schema.name, schema.description.as_deref().unwrap_or("")), 68));
    println!("{}", border);
    println!("|{:68}|", " Fields ");
    println!("{}", "-".repeat(70));

    for (i, field) in schema.fields.iter().enumerate() {
        let marker = if i == 0 { '>' } else { ' ' };
        let req = if field.required { " *" } else { "" };

        // Label
        println!("| {} {}{}{:<55}|", marker, field.label, req, "");

        match field.widget {
            WidgetKind::TextInput => {
                let val = match &field.default {
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                println!("|   +{}+", "-".repeat(64));
                println!("|   | {:<63}|", val);
                println!("|   +{}+", "-".repeat(64));
            }
            WidgetKind::NumberInput => {
                let val = match &field.default {
                    Some(Value::Integer(n)) => n.to_string(),
                    Some(Value::Float(n)) => n.to_string(),
                    _ => "0".to_string(),
                };
                println!("|   [ {} ]{:<53}|", val, "");
            }
            WidgetKind::Checkbox => {
                let checked = match &field.default {
                    Some(Value::Bool(b)) => *b,
                    _ => false,
                };
                let ch = if checked { "x" } else { " " };
                println!("|   [{}] {}{:<49}|", ch, field.label, "");
            }
            WidgetKind::Select => {
                let val = match &field.default {
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                for opt in &field.options {
                    let sel = if opt == val { ">" } else { " " };
                    println!("|     {} {}{:<53}|", sel, opt, "");
                }
            }
            _ => {
                println!("|   [...]{:<57}|", "");
            }
        }

        println!("|{:68}|", "");
    }

    println!("|   ! Input is required{:<45}|", "");
    println!("|{:68}|", "");
    println!("{}", border);
    println!("| {} {:<64}|", "Up/Down Navigate   Enter Edit   Space Toggle   q Quit   F1 Help", "");
    println!("{}", border);

    println!();
    println!("KEYBINDINGS:");
    println!("  Up/Down/j/k   Navigate fields");
    println!("  Enter          Edit field / Confirm");
    println!("  Space          Toggle checkbox");
    println!("  Esc            Cancel / Quit");
    println!("  Tab            Next field");
    println!("  Shift+Tab      Previous field");
    println!();
    println!("Run the real TUI with:");
    println!("  cargo run -p tui-generator-ratatui --example demo");
    println!();
}
