use anyhow::Result;
use crate::shell::{generate_hook, compute_exports};

// ── ven shell hook <shell> ────────────────────────────────────────
pub fn cmd_shell_hook(shell: &str) -> Result<()> {
    // Just print the hook code — user wraps this in eval "$(ven shell hook bash)"
    print!("{}", generate_hook(shell));
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
