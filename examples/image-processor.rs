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
    let cli = Cli::parse_or_tui()?;

    println!("--- Submitted Configuration ---");
    println!("Input:    {:?}", cli.input);
    println!("Output:   {:?}", cli.output);
    println!("Threads:  {}", cli.threads);
    println!("Verbose:  {}", cli.verbose);
    println!("Format:   {}", cli.format);

    Ok(())
}
