use crate::shell::{detect_shell, windows_powershell_profile_paths};
use anyhow::Result;
use colored::Colorize;
use std::io::Write;

// ── ven setup ─────────────────────────────────────────────────────
pub fn cmd_setup() -> Result<()> {
    // FIXED: detect shell properly — Windows always uses PowerShell
    let shell_name = detect_shell();

    println!("\n  {} ven setup", "→".cyan());
    println!("  {} {}", "Shell:".dimmed(), shell_name.bold());

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    const WIN_HOOK_MARKER: &str = "# ven-managed-hook-v2";

    if cfg!(target_os = "windows") {
        // Avoid Out-String (can mangle hook output); load hook lines then Invoke-Expression.
        // Install into every common profile so pwsh, Windows PowerShell, and Cursor/VS Code all load it.
        let hook_block = format!(
            "\n{WIN_HOOK_MARKER}\n\
# ven shell hook — requires `ven` on PATH\n\
$_ven = Get-Command ven -ErrorAction SilentlyContinue\n\
if ($_ven) {{\n\
  $_lines = & $_ven.Source @('shell','hook','powershell') 2>$null\n\
  if ($LASTEXITCODE -eq 0 -and $_lines) {{\n\
    Invoke-Expression ([string]::Join([Environment]::NewLine, @($_lines)))\n\
  }}\n\
}} else {{\n\
  Write-Warning 'ven: not on PATH; skipping ven shell hook'\n\
}}\n"
        );

        let mut any_new = false;
        for rc_file in windows_powershell_profile_paths(&home) {
            let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
            if existing.contains(WIN_HOOK_MARKER) {
                println!(
                    "  {} {}",
                    "✓".green(),
                    format!("Hook already configured in {}", rc_file.display()).dimmed()
                );
                continue;
            }
            if let Some(parent) = rc_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&rc_file)?;
            writeln!(file, "{}", hook_block)?;
            println!(
                "  {} {}",
                "✓".green(),
                format!("Configured {}", rc_file.display()).dimmed()
            );
            any_new = true;
        }
        if !any_new {
            println!(
                "  {} {}",
                "✓".green(),
                "Shell hook is already configured in all target profiles".dimmed()
            );
        }
        println!();
        println!("  {}", "Reload your shell to apply changes.".bold());
        println!(
            "  {} {}",
            "Tip:".dimmed(),
            "run . $PROFILE in this session, or open a new terminal".dimmed()
        );
        println!();
        return Ok(());
    }

    // Unix — bash/zsh/fish
    let rc_file = match shell_name.as_str() {
        "zsh" => home.join(".zshrc"),
        "fish" => home.join(".config").join("fish").join("config.fish"),
        _ => home.join(".bashrc"),
    };
    let hook_line = format!(
        "\n# ven shell hook\neval \"$(ven shell hook {})\"",
        shell_name
    );

    let existing = std::fs::read_to_string(&rc_file).unwrap_or_default();
    if existing.contains("ven shell hook") {
        println!(
            "  {} {}",
            "✓".green(),
            format!("Hook already configured in {}", rc_file.display()).dimmed()
        );
        return Ok(());
    }

    if let Some(parent) = rc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;
    writeln!(file, "{}", hook_line)?;

    println!(
        "  {} {}",
        "✓".green(),
        format!("Configured {}", rc_file.display()).dimmed()
    );
    println!();
    println!("  {}", "Reload your shell to apply changes.".bold());
    println!("  {}", format!("source {}", rc_file.display()).dimmed());
    println!();
    Ok(())
}
