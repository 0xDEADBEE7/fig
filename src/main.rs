pub mod app;
pub mod controls;
pub mod figure;
pub mod modal;
pub mod models;
pub mod settings;

use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;

#[derive(Parser)]
#[command(about = "Interactive terminal graph viewer")]
struct Cli {
    /// JSON input file, or - to read standard input.
    #[arg(default_value = "-")]
    input: PathBuf,
    /// Maximum terminal width to use.
    #[arg(long, default_value_t = 120)]
    width: u16,
    /// Maximum terminal height to use.
    #[arg(long, default_value_t = 40)]
    height: u16,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut input = if cli.input.as_os_str() == "-" {
        String::new()
    } else {
        fs::read_to_string(&cli.input)
            .with_context(|| format!("could not read {}", cli.input.display()))?
    };
    if input.is_empty() {
        io::stdin().read_to_string(&mut input)?;
    }
    let figure = serde_json::from_str(&input).context("input must be a figure JSON document")?;
    app::run(figure, cli.width, cli.height)
}
