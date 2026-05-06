//! Build PATH / toolchain preview from `ven.toml` using the same resolution as `ven shell activate`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::shell::{
    activation_path_overlay, path_for_env_value, resolve_activation_environment,
    ActivationResolve,
};
use anyhow::Result;

/// Print `"PATH should be:"` overlay and other env vars resolved from `project_dir`'s nearest `ven.toml`.
///
/// Caller typically prints `"Detected shell: …"` first (Phase 2).
pub fn print_environment_preview(project_dir: &Path) -> Result<()> {
    match resolve_activation_environment(project_dir)? {
        ActivationResolve::NoToml => {
            eprintln!("ven-launcher: no ven.toml found (walk up from this directory).");
        }
        ActivationResolve::MissingToolchain {
            language,
            install_with,
        } => {
            anyhow::bail!(
                "'{}' {} is missing under ven (install then retry). Hint: ven install {} {}",
                language,
                install_with,
                language,
                install_with
            );
        }
        ActivationResolve::Ready(parts) => {
            println!("PATH should be: {}", activation_path_overlay(&parts));

            if let Some(ref bin) = parts.node_bin_for_path {
                println!("NODE_PATH should be: {}", path_for_env_value(bin));
            }
            if let Some(ref v) = parts.python_resolved {
                println!("VEN_PYTHON_VERSION should be: {v}");
            }
            if let Some(ref root) = parts.go_root_for_env {
                println!("GOROOT should be: {}", path_for_env_value(root));
                if let Some(home) = dirs::home_dir() {
                    println!("GOPATH should be: {}", path_for_env_value(&home.join("go")));
                }
            }
            if let Some(ref root) = parts.rust_root_for_env {
                let r = path_for_env_value(root);
                println!("CARGO_HOME should be: {r}");
                println!("RUSTUP_HOME should be: {r}");
            }
            if let Some(ref home) = parts.java_home_for_env {
                println!("JAVA_HOME should be: {}", path_for_env_value(home));
            }
            if let Some(ref vr) = parts.virtual_env_root {
                println!("VIRTUAL_ENV should be: {}", path_for_env_value(vr));
            }
            if let Some(ref v) = parts.node_resolved {
                println!("VEN_NODE_VERSION should be: {v}");
            }
            if let Some(ref v) = parts.go_resolved {
                println!("VEN_GO_VERSION should be: {v}");
            }
            if let Some(ref v) = parts.rust_resolved {
                println!("VEN_RUST_VERSION should be: {v}");
            }
            if let Some(ref v) = parts.java_resolved {
                println!("VEN_JAVA_VERSION should be: {v}");
            }
            if let Some(ref v) = parts.deno_resolved {
                println!("VEN_DENO_VERSION should be: {v}");
            }
            println!("VEN_TOML should be: {}", parts.toml_normalized);

            let extra: BTreeMap<_, _> = parts.ven_user_env.iter().collect();
            for (key, val) in extra {
                println!("{} should be: {}", key, val);
            }
        }
    }
    Ok(())
}
