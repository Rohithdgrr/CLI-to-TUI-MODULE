use tui_generator::Tui;
use clap::Parser;

#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    name: String,
}

fn main() {
    println!("ok");
}
