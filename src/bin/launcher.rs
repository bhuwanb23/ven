//! Entry point for `ven-launcher` — terminal spawner.

use std::path::PathBuf;

use anyhow::{Context, Result};
use ven::launcher::{detect_shell, env};

fn resolve_project_arg() -> Result<PathBuf> {
    match std::env::args_os().nth(1) {
        Some(s) => {
            let p = PathBuf::from(s);
            Ok(if p.is_absolute() {
                p
            } else {
                std::env::current_dir()
                    .context("cannot determine current directory")?
                    .join(p)
            })
        }
        None => std::env::current_dir().context("cannot determine current directory"),
    }
}

fn main() -> Result<()> {
    println!("Detected shell: {}", detect_shell());

    let dir = resolve_project_arg()?;
    env::print_environment_preview(&dir)
}
