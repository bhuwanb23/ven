//! Build PATH / toolchain preview from `ven.toml` using the same resolution as `ven shell activate`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::shell::{
    activation_path_overlay, path_for_env_value, resolve_activation_environment, ActivationParts,
    ActivationResolve,
};
use anyhow::Result;

/// Same PATH merge as `ven shell activate`: overlay first, then current process `PATH`.
pub fn merged_path_for_child(parts: &ActivationParts) -> String {
    let overlay = activation_path_overlay(parts);
    let base = std::env::var("PATH").unwrap_or_default();
    if overlay.is_empty() {
        base
    } else if cfg!(windows) {
        format!("{overlay};{base}")
    } else {
        format!("{overlay}:{base}")
    }
}

/// Apply merged PATH and toolchain variables to a child process (mirrors `format_activation_shell_script`).
pub fn apply_activation_env(cmd: &mut Command, parts: &ActivationParts) {
    cmd.env("PATH", merged_path_for_child(parts));

    if let Some(ref bin) = parts.node_bin_for_path {
        cmd.env("NODE_PATH", path_for_env_value(bin));
    }
    if let Some(ref v) = parts.node_resolved {
        cmd.env("VEN_NODE_VERSION", v);
    }
    if let Some(ref v) = parts.python_resolved {
        cmd.env("VEN_PYTHON_VERSION", v);
    }
    if let Some(ref v) = parts.go_resolved {
        cmd.env("VEN_GO_VERSION", v);
    }
    if let Some(ref root) = parts.go_root_for_env {
        cmd.env("GOROOT", path_for_env_value(root));
        if let Some(home) = dirs::home_dir() {
            cmd.env("GOPATH", path_for_env_value(&home.join("go")));
        }
    }
    if let Some(ref v) = parts.rust_resolved {
        cmd.env("VEN_RUST_VERSION", v);
    }
    if let Some(ref root) = parts.rust_root_for_env {
        let r = path_for_env_value(root);
        cmd.env("CARGO_HOME", &r);
        cmd.env("RUSTUP_HOME", &r);
    }
    if let Some(ref v) = parts.java_resolved {
        cmd.env("VEN_JAVA_VERSION", v);
    }
    if let Some(ref home) = parts.java_home_for_env {
        cmd.env("JAVA_HOME", path_for_env_value(home));
    }
    if let Some(ref v) = parts.deno_resolved {
        cmd.env("VEN_DENO_VERSION", v);
    }
    if let Some(ref vr) = parts.virtual_env_root {
        cmd.env("VIRTUAL_ENV", path_for_env_value(vr));
    }
    cmd.env("VEN_TOML", &parts.toml_normalized);

    for (key, val) in &parts.ven_user_env {
        if key.eq_ignore_ascii_case("PATH") {
            continue;
        }
        cmd.env(key, val);
    }
}

/// Print `"PATH should be:"` overlay and other env vars resolved from `project_dir`'s nearest `ven.toml`.
///
/// Caller typically prints `"Detected shell: …"` first (Phase 2).
pub fn print_environment_preview(project_dir: &Path) -> Result<()> {
    match resolve_activation_environment(project_dir)? {
        ActivationResolve::NoToml => {
            let start = path_for_env_value(project_dir);
            anyhow::bail!(
                "ven-launcher: no ven.toml found when searching upward from \"{start}\".\n\
                 Hint: use `ven-launcher --show-env` with a directory that contains or is under ven.toml.",
            )
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
