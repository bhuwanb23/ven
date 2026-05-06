//! Entry point for `ven-launcher` — terminal spawner.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use ven::launcher::{detect_shell, env, spawn};

#[derive(Parser)]
#[command(name = "ven-launcher", disable_version_flag = true)]
struct LauncherCli {
    /// Print resolved PATH / env (Phase 3) instead of opening a terminal.
    #[arg(long)]
    show_env: bool,

    /// Project directory (default: current working directory).
    project: Option<PathBuf>,
}

fn resolve_project(cli: &LauncherCli) -> Result<PathBuf> {
    match &cli.project {
        Some(p) => {
            if p.is_absolute() {
                Ok(p.clone())
            } else {
                Ok(std::env::current_dir()
                    .context("cannot determine current directory")?
                    .join(p))
            }
        }
        None => std::env::current_dir().context("cannot determine current directory"),
    }
}

fn main() -> Result<()> {
    let cli = LauncherCli::parse();
    println!("Detected shell: {}", detect_shell());

    let dir = resolve_project(&cli)?;
    if cli.show_env {
        env::print_environment_preview(&dir)?;
    } else {
        spawn::spawn_project_shell(&dir)?;
    }
    Ok(())
}
