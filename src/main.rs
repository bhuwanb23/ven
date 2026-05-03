use anyhow::Result;
use clap::Parser;

mod cli;
mod core;
mod plugins;
mod shell;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::run(cli)
}
