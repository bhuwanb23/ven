use anyhow::Result;
use clap::Parser;
use ven::cli::{run, Cli};

fn main() -> Result<()> {
    run(Cli::parse())
}
