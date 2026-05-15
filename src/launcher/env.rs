//! Build PATH / toolchain preview from `ven.toml` using the same resolution as `ven shell activate`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::launcher::greeting::write_greeting_to_stdout;
use crate::shell::{
    activation_path_overlay, path_for_env_value, resolve_activation_environment, ActivationParts,
    ActivationResolve,
};
use anyhow::Result;

fn launcher_bin_dir() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(path_for_env_value(dir))
}

fn prepend_path(entry: &str, base: &str) -> String {
    if entry.is_empty() {
        return base.to_string();
    }
    if base.is_empty() {
        return entry.to_string();
    }
    if cfg!(windows) {
        format!("{entry};{base}")
    } else {
        format!("{entry}:{base}")
    }
}

/// Ensure the launcher's own folder is on PATH so bundled `ven` is callable.
pub fn merged_path_with_launcher_bin(base: &str) -> String {
    if let Some(bin_dir) = launcher_bin_dir() {
        prepend_path(&bin_dir, base)
    } else {
        base.to_string()
    }
}

/// Same PATH merge as `ven shell activate`: overlay first, then current process `PATH`.
pub fn merged_path_for_child(parts: &ActivationParts) -> String {
    let overlay = activation_path_overlay(parts);
    let base = std::env::var("PATH").unwrap_or_default();
    let merged = if overlay.is_empty() {
        base
    } else if cfg!(windows) {
        format!("{overlay};{base}")
    } else {
        format!("{overlay}:{base}")
    };
    merged_path_with_launcher_bin(&merged)
}

/// Apply merged PATH and toolchain variables to a child process (mirrors `format_activation_shell_script`).
pub fn apply_activation_env(cmd: &mut Command, parts: &ActivationParts) {
    cmd.env("PATH", merged_path_for_child(parts));
    // Propagate the resolved VEN_HOME so every `ven` invocation in the
    // spawned shell sees the same storage root the launcher itself used.
    // Without this, a portable bundle (sibling `.ven/`) silently falls
    // back to `~/.ven` once you're inside the new shell.
    cmd.env(
        "VEN_HOME",
        path_for_env_value(&crate::core::ven_home::ven_home()),
    );

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
    if let Some(ref v) = parts.bun_resolved {
        cmd.env("VEN_BUN_VERSION", v);
    }
    if let Some(ref v) = parts.ruby_resolved {
        cmd.env("VEN_RUBY_VERSION", v);
    }
    if let Some(ref gh) = parts.ruby_gem_home_for_env {
        let ghv = path_for_env_value(gh);
        cmd.env("GEM_HOME", &ghv);
        cmd.env("GEM_PATH", &ghv);
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

/// Apply only launcher portability env (no project runtime resolution).
pub fn apply_launcher_portable_env(cmd: &mut Command) {
    let base = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", merged_path_with_launcher_bin(&base));
    // Even without a ven.toml, child shells should see the launcher's
    // resolved VEN_HOME so subsequent `ven` calls land in the right root.
    cmd.env(
        "VEN_HOME",
        path_for_env_value(&crate::core::ven_home::ven_home()),
    );
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
                 Try from the project folder, or pass a path explicitly, for example:\n\
                   ven-launcher\n\
                   ven-launcher path/to/myapp\n\
                   ven-launcher ./example",
            )
        }
        ActivationResolve::MissingToolchain {
            language,
            install_with,
        } => {
            let start = path_for_env_value(project_dir);
            anyhow::bail!(
                "ven-launcher: required runtime is not installed for this machine.\n\
                 • Language: {language}\n\
                 • Requested in ven.toml: {install_with}\n\
                 • Search started from: {start}\n\
                 Install it, then retry:\n\
                   ven install {language} {install_with}"
            );
        }
        ActivationResolve::Ready(parts) => {
            write_greeting_to_stdout(&parts);
            println!("PATH should be: {}", activation_path_overlay(&parts));
            println!(
                "VEN_HOME should be: {}",
                path_for_env_value(&crate::core::ven_home::ven_home())
            );

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
            if let Some(ref v) = parts.bun_resolved {
                println!("VEN_BUN_VERSION should be: {v}");
            }
            if let Some(ref v) = parts.ruby_resolved {
                println!("VEN_RUBY_VERSION should be: {v}");
            }
            if let Some(ref gh) = parts.ruby_gem_home_for_env {
                let ghv = path_for_env_value(gh);
                println!("GEM_HOME should be: {ghv}");
                println!("GEM_PATH should be: {ghv}");
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
