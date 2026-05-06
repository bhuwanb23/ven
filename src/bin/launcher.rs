//! Entry point for `ven-launcher` — terminal spawner.
//!
//! Usage:
//! - `ven-launcher` — use current directory as the search root for `ven.toml`
//! - `ven-launcher C:\path\to\project` or `ven-launcher .\relative` — search from that path

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ven::launcher::{detect_shell, env, paths, spawn};

#[derive(Parser)]
#[command(
    name = "ven-launcher",
    disable_version_flag = true,
    about = "Open a new terminal with ven runtimes for the nearest ven.toml (walks up from PROJECT or cwd)."
)]
struct LauncherCli {
    /// Print resolved PATH / env instead of opening a terminal.
    #[arg(long)]
    show_env: bool,

    /// Project directory or path to `ven.toml` (default: current working directory).
    #[arg(value_name = "PROJECT")]
    project: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = LauncherCli::parse();
    println!("Detected shell: {}", detect_shell());

    let start = paths::resolve_activation_start_dir(cli.project.as_deref())?;

    if cli.show_env {
        env::print_environment_preview(&start)?;
    } else {
        spawn::spawn_project_shell(&start)?;
    }
    Ok(())
}
