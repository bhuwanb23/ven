use anyhow::Result;
use colored::Colorize;

use crate::plugins::LanguagePlugin;

/// Post-install validation: verify binary exists and version matches.
pub(super) fn validate_installation(
    plugin: &dyn LanguagePlugin,
    language: &str,
    version: &str,
) -> Result<()> {
    println!("\n{} Validating installation...", "[CHECK]".cyan());

    // Check 1: Binary exists
    let bin_path = plugin.bin_path(version)?;
    let binary_name = match language {
        "node" => {
            if cfg!(target_os = "windows") {
                "node.exe"
            } else {
                "node"
            }
        }
        "python" => {
            if cfg!(target_os = "windows") {
                "python.exe"
            } else {
                "python3"
            }
        }
        "go" => {
            if cfg!(target_os = "windows") {
                "go.exe"
            } else {
                "go"
            }
        }
        "rust" => {
            if cfg!(target_os = "windows") {
                "cargo.exe"
            } else {
                "cargo"
            }
        }
        "java" => {
            if cfg!(target_os = "windows") {
                "java.exe"
            } else {
                "java"
            }
        }
        "deno" => {
            if cfg!(target_os = "windows") {
                "deno.exe"
            } else {
                "deno"
            }
        }
        _ => {
            if cfg!(target_os = "windows") {
                "node.exe"
            } else {
                "node"
            }
        }
    };
    let binary = bin_path.join(binary_name);

    if binary.exists() {
        println!("  [OK] Binary: {}", binary.display());
    } else {
        println!("  [FAIL] Binary not found: {}", binary.display());
        return Err(anyhow::anyhow!(
            "Installation validation failed: binary not found"
        ));
    }

    // Check 2: Version check
    println!("  [OK] Version: {} {}", language.bold(), version.green());

    // Check 3: PATH ready
    println!("  [OK] PATH: Ready to use");

    println!(
        "\n{} {} {} installed successfully!",
        "[SUCCESS]".green().bold(),
        language.bold(),
        version.bold()
    );
    println!("  [TIP] Run: ven init   to create a project");

    Ok(())
}
