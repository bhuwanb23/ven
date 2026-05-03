use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use crate::shell::{generate_hook, compute_exports, windows_powershell_profile_paths};

// ── ven shell hook <shell> ────────────────────────────────────────
pub fn cmd_shell_hook(shell: &str) -> Result<()> {
    // Just print the hook code — user wraps this in eval "$(ven shell hook bash)"
    print!("{}", generate_hook(shell));
    Ok(())
}

// ── ven shell install ─────────────────────────────────────────────
// Auto-installs hook into PowerShell profile(s) for permanent activation
pub fn cmd_shell_install() -> Result<()> {
    println!("  {} Installing ven shell hook...", "🔧".cyan());

    const PS_HOOK_MARKER: &str = "# ven shell hook (PowerShell)";

    if cfg!(target_os = "windows") {
        let home = PathBuf::from(std::env::var("USERPROFILE")?);
        let hook_code = generate_hook("powershell");
        let mut wrote: Vec<PathBuf> = Vec::new();

        for profile_path in windows_powershell_profile_paths(&home) {
            if let Some(parent) = profile_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let existing_content = if profile_path.exists() {
                std::fs::read_to_string(&profile_path)?
            } else {
                String::new()
            };

            if existing_content.contains(PS_HOOK_MARKER) {
                continue;
            }

            let mut content = existing_content.clone();
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("\n# ven shell hook - Auto-loads on terminal start\n");
            content.push_str(&hook_code);

            std::fs::write(&profile_path, &content)?;
            wrote.push(profile_path);
        }

        if wrote.is_empty() {
            println!("  {} ven PowerShell hook already present in known profiles", "✅".green());
            for p in windows_powershell_profile_paths(&home) {
                println!("  {} {}", "Profile:".dimmed(), p.display());
            }
        } else {
            println!("  {} ven hook installed successfully!", "✅".green());
            for p in &wrote {
                println!("  {} {}", "Profile:".dimmed(), p.display());
            }
        }
        println!();
        println!("  {} Open a NEW terminal (or run: . $PROFILE in pwsh)", "ℹ️".cyan());
        println!("  {} Cursor/VS Code: use a terminal profile that loads the profile above.", "ℹ️".cyan());
        return Ok(());
    }

    // Unix: single rc file
    let home = std::env::var("HOME")?;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let profile_path = if shell.contains("zsh") {
        format!("{}/.zshrc", home)
    } else {
        format!("{}/.bashrc", home)
    };
    let profile_path = std::path::Path::new(&profile_path);

    if let Some(parent) = profile_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let existing_content = if profile_path.exists() {
        std::fs::read_to_string(profile_path)?
    } else {
        String::new()
    };

    if existing_content.contains("# ven shell hook") {
        println!("  {} ven hook already installed in profile", "✅".green());
        println!("  {} {}", "Profile:".dimmed(), profile_path.display());
        return Ok(());
    }

    let hook_code = generate_hook("bash");
    let mut content = existing_content.clone();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n# ven shell hook - Auto-loads on terminal start\n");
    content.push_str(&hook_code);

    std::fs::write(profile_path, &content)?;

    println!("  {} ven hook installed successfully!", "✅".green());
    println!("  {} {}", "Profile:".dimmed(), profile_path.display());
    println!();
    println!("  {} The hook will auto-load in NEW terminals", "ℹ️".cyan());
    println!("  {} Close and reopen your terminal to activate", "ℹ️".cyan());
    println!();
    println!("  {} To activate in current terminal, run:", "💡".yellow());
    println!("     {}", format!("source {}", profile_path.display()).dimmed());

    Ok(())
}

// ── ven shell activate <dir> ──────────────────────────────────────
#[allow(non_snake_case)]
pub fn cmd_shell_activate(dir: &str) -> Result<()> {
    let path = std::path::Path::new(dir);
    
    // Check if directory exists
    if !path.exists() {
        anyhow::bail!("Directory not found: {}", path.display());
    }
    
    match compute_exports(path)? {
        Some(exports) => {
            // Output the exports (stdout only — shell hooks eval this)
            print!("{}", exports);
        }
        None => {
            // No ven.toml in this tree: exit 0 with no stdout so hooks restore PATH
            // (interactive users can run `ven status` / `ven init` for guidance)
        }
    }
    Ok(())
}
