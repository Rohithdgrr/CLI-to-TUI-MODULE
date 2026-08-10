use std::path::PathBuf;
use tui_generator::Tui;
use clap::Parser;

#[derive(Parser, Tui)]
#[tui(title = "Image Processor")]
#[tui(description = "Configure image processing options")]
struct Cli {
    /// Input image path
    #[arg(short, long)]
    input: PathBuf,

    /// Output image path
    #[arg(short, long)]
    output: PathBuf,

    /// Number of processing threads
    #[arg(long)]
    #[tui(default = "4")]
    threads: usize,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output format
    #[arg(long)]
    #[tui(options = "png,jpg,webp,gif")]
    format: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Test mode selection
    if args.contains(&"--test".to_string()) {
        run_tests()?;
    } else {
        let cli = Cli::parse_or_tui()?;
        println!("--- Submitted Configuration ---");
        println!("Input:    {:?}", cli.input);
        println!("Output:   {:?}", cli.output);
        println!("Threads:  {}", cli.threads);
        println!("Verbose:  {}", cli.verbose);
        println!("Format:   {}", cli.format);
    }

    Ok(())
}

fn run_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TUI Generator Test Suite ===\n");

    // Test 1: Schema generation
    print!("Test 1: Schema generation... ");
    let schema = Cli::schema();
    assert_eq!(schema.name, "Image Processor");
    assert_eq!(schema.description.as_deref(), Some("Configure image processing options"));
    assert_eq!(schema.fields.len(), 5);
    println!("PASS");

    // Test 2: Field names
    print!("Test 2: Field names... ");
    let names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["input", "output", "threads", "verbose", "format"]);
    println!("PASS");

    // Test 3: Field labels (auto title-cased)
    print!("Test 3: Field labels... ");
    let labels: Vec<&str> = schema.fields.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(labels, vec!["Input", "Output", "Threads", "Verbose", "Format"]);
    println!("PASS");

    // Test 4: Field descriptions from doc comments
    print!("Test 4: Field descriptions... ");
    assert_eq!(schema.fields[0].description.as_deref(), Some("Input image path"));
    assert_eq!(schema.fields[2].description.as_deref(), Some("Number of processing threads"));
    println!("PASS");

    // Test 5: Widget types
    print!("Test 5: Widget types... ");
    use tui_generator::core::widget::WidgetKind;
    assert_eq!(schema.fields[0].widget, WidgetKind::PathInput);   // PathBuf
    assert_eq!(schema.fields[2].widget, WidgetKind::NumberInput); // usize
    assert_eq!(schema.fields[3].widget, WidgetKind::Checkbox);    // bool
    assert_eq!(schema.fields[4].widget, WidgetKind::TextInput);   // String
    println!("PASS");

    // Test 6: Required fields
    print!("Test 6: Required fields... ");
    assert!(!schema.fields[0].required);  // input - PathBuf has type default
    assert!(!schema.fields[1].required);  // output - PathBuf has type default
    assert!(!schema.fields[2].required);  // threads - has default
    assert!(!schema.fields[3].required);  // verbose - bool has default false
    assert!(!schema.fields[4].required);  // format - String has type default
    println!("PASS");

    // Test 7: Default values
    print!("Test 7: Default values... ");
    assert_eq!(schema.fields[2].default, Some(tui_generator::core::value::Value::Integer(4)));
    assert_eq!(schema.fields[3].default, Some(tui_generator::core::value::Value::Bool(false)));
    println!("PASS");

    // Test 8: Options for select fields
    print!("Test 8: Field options... ");
    assert_eq!(schema.fields[4].options, vec!["png", "jpg", "webp", "gif"]);
    println!("PASS");

    // Test 9: to_values roundtrip
    print!("Test 9: to_values roundtrip... ");
    let cli = Cli {
        input: PathBuf::from("./test.png"),
        output: PathBuf::from("./out.png"),
        threads: 8,
        verbose: true,
        format: "jpg".to_string(),
    };
    let values = cli.to_values();
    assert_eq!(values.get("input"), Some(&tui_generator::core::value::Value::Path(PathBuf::from("./test.png"))));
    assert_eq!(values.get("threads"), Some(&tui_generator::core::value::Value::Integer(8)));
    assert_eq!(values.get("verbose"), Some(&tui_generator::core::value::Value::Bool(true)));
    assert_eq!(values.get("format"), Some(&tui_generator::core::value::Value::String("jpg".to_string())));
    println!("PASS");

    // Test 10: from_values roundtrip
    print!("Test 10: from_values roundtrip... ");
    let restored = Cli::from_values(&values)?;
    assert_eq!(restored.input, PathBuf::from("./test.png"));
    assert_eq!(restored.output, PathBuf::from("./out.png"));
    assert_eq!(restored.threads, 8);
    assert_eq!(restored.verbose, true);
    assert_eq!(restored.format, "jpg");
    println!("PASS");

    // Test 11: CLI parse works
    print!("Test 11: CLI parse works... ");
    let args = vec!["test", "--input", "./a.txt", "--output", "./b.txt", "--threads", "16", "--verbose", "--format", "webp"];
    let cli = Cli::try_parse_from(args)?;
    assert_eq!(cli.input, PathBuf::from("./a.txt"));
    assert_eq!(cli.output, PathBuf::from("./b.txt"));
    assert_eq!(cli.threads, 16);
    assert!(cli.verbose);
    assert_eq!(cli.format, "webp");
    println!("PASS");

    println!("\n=== All 11 tests PASSED ===");
    Ok(())
}
