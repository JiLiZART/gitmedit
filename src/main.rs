use std::path::PathBuf;

use clap::Parser;

mod context;

#[derive(Parser, Debug)]
#[command(name = "gitmedit", about = "Fast, distraction-free git editor")]
struct Cli {
    /// Path to the file to edit (provided by git)
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let ctx = context::detect_context(&cli.path);
    println!("context: {:?}", ctx);
}
