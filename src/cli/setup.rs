use anyhow::Result;
use colored::Colorize;
use std::io::Write;
use crate::shell::{detect_shell};

// ── ven setup ─────────────────────────────────────────────────────
pub fn cmd_setup() -> Result<()> {
    // FIXED: detect shell properly — Windows always uses PowerShell
    let shell_name = detect_shell();

    println!("\n  {} ven setup", "→".cyan());
    println!("  Detected shell: {}", shell_name.bold());

    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    // FIXED: Windows writes to PowerShell $PROFILE, not ~/.bashrc
    let (rc_file, hook_line) = if cfg!(target_os = "windows") {
        // PowerShell profile location
        let profile = home
            .join("Documents")
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1");
        let line = "\n# ven shell hook\nInvoke-Expression (& ven shell hook powershell | Out-String)".to_string();
        (profile, line)
    } else {
        // Unix — bash/zsh/fish
        let rc = match shell_name.as_str() {
            "zsh"  => home.join(".zshrc"),
            "fish" => home.join(".config").join("fish").join("config.fish"),
            _      => home.join(".bashrc"),
        };
        let line = format!("\n# ven shell hook\neval \"$(ven shell hook {})\""  , shell_name);
        (rc, line)
    };

    // Check if already installed
    let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
    if existing.contains("ven shell hook") {
        println!("  {} Shell hook already installed in {}", "✓".green(), rc_file.display());
        return Ok(());
    }

    // Create parent dirs if needed (PowerShell profile dir may not exist)
    if let Some(parent) = rc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Append hook line to rc file
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;
    writeln!(file, "{}", hook_line)?;

    println!("  {} Written to {}", "✓".green(), rc_file.display());
    println!();

    if cfg!(target_os = "windows") {
        println!("  Restart PowerShell or run:");
        println!("  {}", ". $PROFILE".bold());
    } else {
        println!("  Restart your shell or run:");
        println!("  {}", format!("source {}", rc_file.display()).bold());
    }
    println!();
    Ok(())
}
