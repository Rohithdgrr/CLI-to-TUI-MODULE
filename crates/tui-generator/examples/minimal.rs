use tui_generator::Tui;
use clap::Parser;

#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_or_tui()?;
    println!("Hello, {}!", cli.name);
    Ok(())
}
