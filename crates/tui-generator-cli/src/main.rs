use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tui_generator_core::schema::{Field, TuiSchema};
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;
use tui_generator_ratatui::RatatuiRenderer;

#[derive(Parser)]
#[command(name = "tui-gen")]
#[command(about = "Universal TUI Generator — turn any CLI into a TUI")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a TUI form from field definitions
    Run {
        /// Form title
        #[arg(short, long, default_value = "TUI Form")]
        title: String,

        /// Field definitions: name:type:default:section:options
        /// Examples:
        ///   --field "input:path:./input.png:Source"
        ///   --field "threads:number:4:Advanced"
        ///   --field "format:select:png:Output:png,jpg,webp"
        ///   --field "verbose:bool:false:Output"
        #[arg(short, long, required = true)]
        field: Vec<String>,

        /// Run a command with the collected args instead of printing
        #[arg(short, long)]
        exec: Option<String>,
    },

    /// Quick example/demo TUI
    Demo,
}

fn main() {
    // If no arguments are provided, default to running the demo
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() == 1 {
        // No subcommand provided - create a CLI instance that runs Demo
        Cli {
            command: Commands::Demo,
        }
    } else {
        Cli::parse()
    };

    match cli.command {
        Commands::Run { title, field, exec } => {
            let schema = parse_fields(&title, &field);
            match RatatuiRenderer::new().run_tui(&schema) {
                Ok(state) => {
                    let args = state.to_cli_args(&schema);
                    if args.is_empty() {
                        println!("(no values set)");
                        std::process::exit(0);
                    }

                    if let Some(cmd) = exec {
                        println!("Running: {} {}", cmd, args.join(" "));
                        let status = std::process::Command::new("cmd")
                            .arg("/C")
                            .arg(&cmd)
                            .args(&args)
                            .status()
                            .expect("failed to execute");
                        std::process::exit(status.code().unwrap_or(1));
                    } else {
                        println!("{}", args.join(" "));
                    }
                }
                Err(e) => {
                    eprintln!("Cancelled or error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Demo => run_demo(),
    }
}

fn parse_fields(title: &str, field_defs: &[String]) -> TuiSchema {
    let mut fields = Vec::new();

    for def in field_defs {
        let parts: Vec<&str> = def.split(':').collect();
        let name = parts[0].to_string();
        let ty = parts.get(1).unwrap_or(&"text");
        let default_val = parts.get(2).unwrap_or(&"").to_string();
        let section = parts.get(3).map(|s| s.to_string());
        let options: Vec<String> = parts
            .get(4)
            .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
            .unwrap_or_default();

        let (widget, default) = parse_type(ty, &default_val);
        let required = default.is_none() && default_val.is_empty();

        fields.push(Field {
            name: name.clone(),
            label: to_title_case(&name),
            description: None,
            required,
            default,
            widget,
            constraints: vec![],
            options,
            skip: false,
            section,
            readonly: false,
        });
    }

    TuiSchema {
        name: title.to_string(),
        description: Some("Configure options below".to_string()),
        fields,
        subcommands: vec![],
    }
}

fn parse_type(ty: &str, default: &str) -> (WidgetKind, Option<Value>) {
    match ty {
        "text" | "string" | "str" => (
            WidgetKind::TextInput,
            if default.is_empty() {
                None
            } else {
                Some(Value::String(default.to_string()))
            },
        ),
        "password" => (
            WidgetKind::PasswordInput,
            if default.is_empty() {
                None
            } else {
                Some(Value::String(default.to_string()))
            },
        ),
        "path" | "file" => (
            WidgetKind::PathInput,
            if default.is_empty() {
                None
            } else {
                Some(Value::Path(PathBuf::from(default)))
            },
        ),
        "number" | "int" | "usize" | "u32" | "i32" => (
            WidgetKind::NumberInput,
            if default.is_empty() {
                Some(Value::Integer(0))
            } else {
                Some(Value::Integer(default.parse().unwrap_or(0)))
            },
        ),
        "float" | "f32" | "f64" => (
            WidgetKind::NumberInput,
            if default.is_empty() {
                Some(Value::Float(0.0))
            } else {
                Some(Value::Float(default.parse().unwrap_or(0.0)))
            },
        ),
        "bool" | "checkbox" | "flag" => (
            WidgetKind::Checkbox,
            Some(Value::Bool(default == "true" || default == "1")),
        ),
        "select" | "choice" | "enum" => (
            WidgetKind::Select,
            if default.is_empty() {
                None
            } else {
                Some(Value::String(default.to_string()))
            },
        ),
        "multiselect" | "multi" => (
            WidgetKind::MultiSelect,
            Some(Value::List(
                default
                    .split(',')
                    .map(|s| Value::String(s.trim().to_string()))
                    .collect(),
            )),
        ),
        _ => (
            WidgetKind::TextInput,
            if default.is_empty() {
                None
            } else {
                Some(Value::String(default.to_string()))
            },
        ),
    }
}

fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            result.push(' ');
            capitalize = true;
        } else if capitalize {
            result.extend(c.to_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn run_demo() {
    let schema = TuiSchema {
        name: "Image Processor".to_string(),
        description: Some("Convert any webpage into a desktop app".to_string()),
        fields: vec![
            Field {
                name: "input".into(),
                label: "Input Path".into(),
                description: Some("Source image to process".into()),
                required: true,
                default: Some(Value::String("./input.png".into())),
                widget: WidgetKind::PathInput,
                constraints: vec![],
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
                default: Some(Value::String("./output.jpg".into())),
                widget: WidgetKind::PathInput,
                constraints: vec![],
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
                default: Some(Value::Integer(1200)),
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
                default: Some(Value::Integer(800)),
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
                default: Some(Value::Bool(false)),
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
                default: Some(Value::String("png".into())),
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
                default: Some(Value::Bool(false)),
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

    match RatatuiRenderer::new().run_tui(&schema) {
        Ok(state) => {
            println!("{}", state.to_cli_args(&schema).join(" "));
        }
        Err(_) => std::process::exit(1),
    }
}