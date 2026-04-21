use anyhow::Result;
use colored::Colorize;
use crate::shell::{generate_hook, compute_exports};

// ── ven shell hook <shell> ────────────────────────────────────────
pub fn cmd_shell_hook(shell: &str) -> Result<()> {
    // Just print the hook code — user wraps this in eval "$(ven shell hook bash)"
    print!("{}", generate_hook(shell));
    Ok(())
}

// ── ven shell install ─────────────────────────────────────────────
// Auto-installs hook into PowerShell profile for permanent activation
pub fn cmd_shell_install() -> Result<()> {
    println!("  {} Installing ven shell hook...", "🔧".cyan());
    
    // Get PowerShell profile path
    let profile_path = if cfg!(target_os = "windows") {
        // Windows PowerShell profile
        let home = std::env::var("USERPROFILE")?;
        format!("{}\\Documents\\WindowsPowerShell\\Microsoft.PowerShell_profile.ps1", home)
    } else {
        // Unix bash/zsh profile
        let home = std::env::var("HOME")?;
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        if shell.contains("zsh") {
            format!("{}/.zshrc", home)
        } else {
            format!("{}/.bashrc", home)
        }
    };
    
    let profile_path = std::path::Path::new(&profile_path);
    
    // Create parent directory if needed
    if let Some(parent) = profile_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Check if hook already installed
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
    
    // Get the hook code
    let shell_type = if cfg!(target_os = "windows") {
        "powershell"
    } else {
        "bash"
    };
    let hook_code = generate_hook(shell_type);
    
    // Append hook to profile
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
    println!("     {}", format!(". {}", profile_path.display()).dimmed());
    
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
            // Output the exports
            print!("{}", exports);
        }
        None => {
            // No ven.toml found - show helpful error
            eprintln!("Error: No ven.toml found in {} or parent directories", path.display());
            eprintln!();
            eprintln!("Initialize: ven init");
            eprintln!("Or specify directory: ven shell activate /path/to/project");
            std::process::exit(1);
        }
    }
    Ok(())
}
