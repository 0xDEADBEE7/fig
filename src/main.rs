use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

mod session;

use anyhow::{Context, Result};
use clap::Parser;
use fig::{RenderOptions, from_json, render};
use std::io::IsTerminal;

#[derive(Debug, Parser)]
#[command(version, about = "Render JSON figures in the terminal")]
struct Cli {
    /// JSON file to read, or - for standard input
    #[arg(default_value = "-")]
    input: PathBuf,

    /// Output width, or maximum interactive width, in terminal columns
    #[arg(short, long, default_value_t = 80)]
    width: usize,

    /// Output height, or maximum interactive height, in terminal rows
    #[arg(short = 'H', long, default_value_t = 24)]
    height: usize,

    /// Force-layout iterations (higher can improve crowded graphs)
    #[arg(long, default_value_t = 300)]
    iterations: usize,

    /// Disable ANSI colors
    #[arg(long)]
    no_color: bool,

    /// Minimum x value displayed by figures
    #[arg(long, allow_hyphen_values = true)]
    x_min: Option<f64>,

    /// Maximum x value displayed by figures
    #[arg(long, allow_hyphen_values = true)]
    x_max: Option<f64>,

    /// Open a redraw-in-place session (input must be a file)
    #[arg(short = 'i', long)]
    interactive: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let input = if cli.input.as_os_str() == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .context("failed to read standard input")?;
        input
    } else {
        fs::read_to_string(&cli.input)
            .with_context(|| format!("failed to read {}", cli.input.display()))?
    };
    let figure = from_json(&input).context("invalid figure JSON")?;
    if cli.interactive {
        anyhow::ensure!(
            cli.input.as_os_str() != "-",
            "interactive mode needs a file input because standard input is used for key events"
        );
        return session::run(
            &figure,
            cli.x_min,
            cli.x_max,
            cli.width,
            cli.height,
            !cli.no_color,
        );
    }
    println!(
        "{}",
        render(
            &figure,
            RenderOptions {
                width: cli.width,
                height: cli.height,
                iterations: cli.iterations,
                color: !cli.no_color && io::stdout().is_terminal(),
                x_min: cli.x_min,
                x_max: cli.x_max,
                y_min: None,
                y_max: None,
                selected_index: None,
                trim_output: true,
            }
        )?
    );
    Ok(())
}
