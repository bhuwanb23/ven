use crate::shell::{compute_exports, generate_hook};
use anyhow::Result;

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
    match compute_exports(path)? {
        Some(exports) => print!("{}", exports),
        None => {} // no ven.toml = print nothing = no eval
    }
    Ok(())
}
