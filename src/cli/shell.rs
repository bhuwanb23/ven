use crate::shell::{
    generate_hook, try_compute_exports, windows_powershell_profile_paths, ComputeExportsOutcome,
};
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

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
            println!(
                "  {} ven PowerShell hook already present in known profiles",
                "✅".green()
            );
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
        println!(
            "  {} Open a NEW terminal (or run: . $PROFILE in pwsh)",
            "ℹ️".cyan()
        );
        println!(
            "  {} Cursor/VS Code: use a terminal profile that loads the profile above.",
            "ℹ️".cyan()
        );
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
    println!(
        "  {} Close and reopen your terminal to activate",
        "ℹ️".cyan()
    );
    println!();
    println!("  {} To activate in current terminal, run:", "💡".yellow());
    println!(
        "     {}",
        format!("source {}", profile_path.display()).dimmed()
    );

    Ok(())
}

// ── ven shell activate <dir> ──────────────────────────────────────
#[allow(non_snake_case)]
pub fn cmd_shell_activate(dir: &str) -> Result<()> {
    let path = std::path::Path::new(dir);

    if !path.exists() {
        anyhow::bail!("Directory not found: {}", path.display());
    }

    match try_compute_exports(path)? {
        ComputeExportsOutcome::NoToml => Ok(()),
        ComputeExportsOutcome::Success(exports) => {
            print!("{}", exports);
            hint_shell_activate_apply_if_tty();
            Ok(())
        }
        ComputeExportsOutcome::MissingToolchain {
            language,
            install_with,
        } => {
            let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
            if interactive {
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "{} {} is required by ven.toml but not installed. Install it now?",
                        language, install_with
                    ))
                    .default(true)
                    .interact()?
                {
                    crate::cli::install::cmd_install(&language, &install_with)?;
                    match try_compute_exports(path)? {
                        ComputeExportsOutcome::Success(exports) => {
                            print!("{}", exports);
                            hint_shell_activate_apply_if_tty();
                        }
                        ComputeExportsOutcome::MissingToolchain { .. } => {
                            anyhow::bail!(
                                "Install finished but activation still failed. Try: ven shell activate {}",
                                path.display()
                            );
                        }
                        ComputeExportsOutcome::NoToml => {}
                    }
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Required runtime not installed.\n\nInstall: ven install {} {}",
                    language,
                    install_with
                ))
            }
        }
    }
}

/// `ven shell activate` only **prints** shell code; the parent process cannot apply it.
/// When the user runs it directly (TTY → TTY), explain; when piped to `iex`/`eval`, no hint.
/// Print snippets to revert PATH overlay from hooks (evaluate in same terminal session).
pub fn cmd_shell_deactivate() -> Result<()> {
    #[cfg(target_os = "windows")]
    print!(
        r#"if ($null -ne $global:VEN_ORIGINAL_PATH) {{
    $env:PATH = $global:VEN_ORIGINAL_PATH
}}
Remove-Item Env:VEN_NODE_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_PYTHON_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_GO_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_RUST_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_JAVA_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_DENO_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_BUN_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:VEN_RUBY_VERSION -ErrorAction SilentlyContinue
Remove-Item Env:GEM_HOME -ErrorAction SilentlyContinue
Remove-Item Env:GEM_PATH -ErrorAction SilentlyContinue
Remove-Item Env:VEN_TOML -ErrorAction SilentlyContinue
Remove-Item Env:VIRTUAL_ENV -ErrorAction SilentlyContinue
Remove-Item Env:GOROOT -ErrorAction SilentlyContinue
Remove-Item Env:GOPATH -ErrorAction SilentlyContinue
Remove-Item Env:CARGO_HOME -ErrorAction SilentlyContinue
Remove-Item Env:RUSTUP_HOME -ErrorAction SilentlyContinue
Remove-Item Env:JAVA_HOME -ErrorAction SilentlyContinue
$env:VEN_SKIP_PROJECT_VENV = '1'
# Force next __ven_activate to re-run (same-dir cache would skip activate and leave stray venv on PATH)
$global:VEN_LAST_DIR = $null
$global:VEN_LAST_TOML_SIG = $null
$global:VEN_LAST_ACTIVATE_WARN = $null

"#
    );
    #[cfg(not(target_os = "windows"))]
    print!(
        r#"if test -n "$__VEN_ORIGINAL_PATH"; then export PATH="$__VEN_ORIGINAL_PATH"; fi
unset VEN_NODE_VERSION 2>/dev/null || true
unset VEN_PYTHON_VERSION 2>/dev/null || true
unset VEN_GO_VERSION 2>/dev/null || true
unset VEN_RUST_VERSION 2>/dev/null || true
unset VEN_JAVA_VERSION 2>/dev/null || true
unset VEN_DENO_VERSION 2>/dev/null || true
unset VEN_BUN_VERSION 2>/dev/null || true
unset VEN_RUBY_VERSION 2>/dev/null || true
unset GEM_HOME 2>/dev/null || true
unset GEM_PATH 2>/dev/null || true
unset VEN_TOML 2>/dev/null || true
unset VIRTUAL_ENV 2>/dev/null || true
unset GOROOT 2>/dev/null || true
unset GOPATH 2>/dev/null || true
unset CARGO_HOME 2>/dev/null || true
unset RUSTUP_HOME 2>/dev/null || true
unset JAVA_HOME 2>/dev/null || true
export VEN_SKIP_PROJECT_VENV=1
unset __VEN_LAST_DIR 2>/dev/null || true
unset __VEN_LAST_TOML_SIG 2>/dev/null || true

"#
    );
    hint_shell_deactivate_apply_if_tty();
    Ok(())
}

/// Same as [`cmd_shell_activate`]; alias for workflows that expect top-level `ven use`.
#[allow(non_snake_case)]
pub fn cmd_use(dir: &str) -> Result<()> {
    cmd_shell_activate(dir)
}

fn hint_shell_deactivate_apply_if_tty() {
    if !(io::stdout().is_terminal() && io::stderr().is_terminal()) {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = writeln!(
            io::stderr(),
            "{}",
            r##"ven: Printed lines above are not executed automatically. Apply with:  iex ((ven deactivate) -join "`n")"##
                .dimmed()
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = writeln!(
            io::stderr(),
            "{}",
            "ven: Printed lines above are not executed automatically. Apply with:  eval \"$(ven deactivate)\"".dimmed()
        );
    }
}

fn hint_shell_activate_apply_if_tty() {
    if !(io::stdout().is_terminal() && io::stderr().is_terminal()) {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = writeln!(
            io::stderr(),
            "{}",
            r##"ven: The lines above are not executed automatically. Apply them with:  iex ((ven shell activate $PWD) -join "`n")  or:  ven-use"##
                .dimmed()
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = writeln!(
            io::stderr(),
            "{}",
            "ven: The lines above are not executed automatically. Apply with:  eval \"$(ven shell activate .)\"  or:  ven-use"
                .dimmed()
        );
    }
}
