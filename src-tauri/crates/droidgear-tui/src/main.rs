mod app;
mod editor;
mod tui;
mod ui;

use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "droidgear-tui")]
#[command(version)]
#[command(about = "DroidGear TUI (headless terminal UI)")]
struct Cli {
    /// Override $HOME for reading/writing config files (useful in containers/tests)
    #[arg(long)]
    home: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let home_dir = match cli.home {
        Some(p) => p,
        None => dirs::home_dir().context("Failed to determine $HOME")?,
    };

    let mut app = app::App::new(home_dir);
    tui::run(&mut app)
}

