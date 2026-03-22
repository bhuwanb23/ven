mod cli;
mod core;
mod plugins;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
