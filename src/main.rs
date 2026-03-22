use clap::Parser;
use anyhow::Result;

mod cli;
mod core;
mod plugins;
mod shell;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::run(cli)
}
