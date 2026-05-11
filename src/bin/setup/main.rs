//! `ven-setup` installer entry point.
//!
//! Cross-platform: Windows (User no-admin / System UAC) and Unix (user `~/.ven/bin` /
//! system `/usr/local/bin` with sudo). Platform-specific install code lives in
//! [`windows`] and [`unix`]; shared CLI, banner, mode prompt, and binary-embedding
//! helpers live in [`common`].

use anyhow::Result;
use clap::Parser;

mod common;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

fn main() -> Result<()> {
    let cli = common::SetupCli::parse();
    common::print_banner(cli.elevated_child);
    let mode = common::resolve_mode(&cli)?;

    #[cfg(windows)]
    {
        windows::run(cli, mode)
    }
    #[cfg(unix)]
    {
        unix::run(cli, mode)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (cli, mode);
        anyhow::bail!("ven-setup is supported on Windows and Unix-like systems only.");
    }
}
