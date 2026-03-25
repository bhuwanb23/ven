use anyhow::Result;
use std::path::Path;

use crate::core::{find_ven_toml, parse_ven_toml, resolve_node_version};
use crate::plugins::{LanguagePlugin, NodePlugin};

// ── Generate hook script ────────────────────────────────────────────
// Returns the shell code to add to ~/.bashrc or ~/.zshrc
// User runs:  eval "$(ven shell hook bash)"  in their rc file

pub fn generate_hook(shell: &str) -> String {
    match shell {
        "bash" | "zsh" => bash_zsh_hook(),
        "fish" => fish_hook(),
        other => format!("echo 'ven: unknown shell: {}' >&2", other),
    }
}

fn bash_zsh_hook() -> String {
    r#"
# ven shell hook
__ven_activate() {
    local exports
    exports=$(ven shell activate "$PWD" 2>/dev/null)
    if [ -n "$exports" ]; then
        eval "$exports"
    fi
}
cd() { builtin cd "$@" && __ven_activate; }
__ven_activate  # activate on shell start
"#
    .to_string()
}

fn fish_hook() -> String {
    r#"
# ven shell hook (fish)
function __ven_activate --on-variable PWD
    set exports (ven shell activate "$PWD" 2>/dev/null)
    if test -n "$exports"
        eval $exports
    end
end
__ven_activate  # activate on shell start
"#
    .to_string()
}

// ── Compute exports for a directory ─────────────────────────────────
// Called by: ven shell activate <dir>
// Reads ven.toml in dir (or parent), figures out which Node version
// is needed, and PRINTS shell export commands as text.
// The hook uses eval to run them in the current shell.

#[allow(non_snake_case)]
pub fn compute_exports(dir: &Path) -> Result<Option<String>> {
    // Find nearest ven.toml
    let toml_path = match find_ven_toml(dir) {
        Some(p) => p,
        None => return Ok(None), // no ven.toml = no activation, print nothing
    };

    let config = parse_ven_toml(&toml_path)?;

    // Get the node version spec from config
    let node_spec = &config.runtime.node;

    // Resolve alias ("lts", "20") to concrete version ("20.11.0")
    let plugin = NodePlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = resolve_node_version(&node_spec, &installed)?;

    // Get the bin/ path for this version
    let bin_path = plugin.bin_path(&resolved)?;

    // Build the exports string — this is what eval will execute in the shell
    let exports = format!(
        r#"export PATH="{bin}:$PATH"
export VEN_NODE_VERSION="{ver}"
export VEN_TOML="{toml}"
"#,
        bin = bin_path.display(),
        ver = resolved,
        toml = toml_path.display(),
    );

    // Also export env vars declared in [env] section
    if !config.env.is_empty() {
        let mut full_exports = exports;
        for (key, val) in &config.env {
            full_exports.push_str(&format!("export {}=\"{}\"\n", key, val));
        }
        return Ok(Some(full_exports));
    }

    Ok(Some(exports))
}
