use crate::core::config::VenConfig;
use anyhow::Result;
use serde_json::json;
use std::path::Path;

use super::helpers::*;

pub(super) fn output_json_status(
    cwd: &Path,
    toml_path: &Path,
    config: &VenConfig,
    verbose: bool,
) -> Result<()> {
    let mut runtime_info = json!({});
    if !config.runtime.node.is_empty() {
        let node_spec = &config.runtime.node;
        let resolved = resolve_version_for_display(node_spec)?;
        let installed = is_version_installed(node_spec);
        runtime_info["node"] = json!({
            "version_required": node_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.python.is_empty() {
        let py_spec = &config.runtime.python;
        let resolved = resolve_python_for_display(py_spec)?;
        let installed = is_python_installed(py_spec);
        runtime_info["python"] = json!({
            "version_required": py_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.go.is_empty() {
        let go_spec = &config.runtime.go;
        let resolved = resolve_go_for_display(go_spec)?;
        let installed = is_go_installed(go_spec);
        runtime_info["go"] = json!({
            "version_required": go_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.rust.is_empty() {
        let rust_spec = &config.runtime.rust;
        let resolved = resolve_rust_for_display(rust_spec)?;
        let installed = is_rust_installed(rust_spec);
        runtime_info["rust"] = json!({
            "version_required": rust_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.java.is_empty() {
        let java_spec = &config.runtime.java;
        let resolved = resolve_java_for_display(java_spec)?;
        let installed = is_java_installed(java_spec);
        runtime_info["java"] = json!({
            "version_required": java_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.deno.is_empty() {
        let deno_spec = &config.runtime.deno;
        let resolved = resolve_deno_for_display(deno_spec)?;
        let installed = is_deno_installed(deno_spec);
        runtime_info["deno"] = json!({
            "version_required": deno_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.bun.is_empty() {
        let bun_spec = &config.runtime.bun;
        let resolved = resolve_bun_for_display(bun_spec)?;
        let installed = is_bun_installed(bun_spec);
        runtime_info["bun"] = json!({
            "version_required": bun_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }
    if !config.runtime.ruby.is_empty() {
        let ruby_spec = &config.runtime.ruby;
        let resolved = resolve_ruby_for_display(ruby_spec)?;
        let installed = is_ruby_installed(ruby_spec);
        runtime_info["ruby"] = json!({
            "version_required": ruby_spec,
            "version_resolved": resolved,
            "installed": installed
        });
    }

    // Build package list
    let mut pkg_list = Vec::new();
    let mut installed_count = 0;

    for (name, version) in &config.packages {
        let deno_without_npm_semantics = !config.runtime.deno.is_empty()
            && config.runtime.ruby.is_empty()
            && config.runtime.node.is_empty()
            && config.runtime.python.is_empty()
            && config.runtime.go.is_empty()
            && config.runtime.rust.is_empty()
            && config.runtime.java.is_empty();
        let ruby_without_npm_semantics = !config.runtime.ruby.is_empty()
            && config.runtime.node.is_empty()
            && config.runtime.python.is_empty()
            && config.runtime.go.is_empty()
            && config.runtime.rust.is_empty()
            && config.runtime.java.is_empty()
            && config.runtime.deno.is_empty();
        let bun_runtime = !config.runtime.bun.is_empty()
            && config.runtime.node.is_empty()
            && config.runtime.python.is_empty()
            && config.runtime.go.is_empty()
            && config.runtime.rust.is_empty()
            && config.runtime.java.is_empty()
            && config.runtime.deno.is_empty()
            && config.runtime.ruby.is_empty();

        let is_installed = if deno_without_npm_semantics || ruby_without_npm_semantics {
            false
        } else if bun_runtime {
            is_package_installed(name)
        } else if !config.runtime.python.is_empty()
            && config.runtime.node.is_empty()
            && config.runtime.go.is_empty()
            && config.runtime.rust.is_empty()
            && config.runtime.java.is_empty()
            && config.runtime.deno.is_empty()
            && config.runtime.ruby.is_empty()
        {
            is_python_package_installed(name)
        } else {
            is_package_installed(name)
        };
        let mut pkg_info = json!({
            "name": name,
            "version_declared": version,
            "installed": is_installed
        });

        if is_installed {
            installed_count += 1;
            if let Ok(installed_ver) = get_installed_package_version(name) {
                pkg_info["version_installed"] = json!(installed_ver);

                if verbose {
                    // Get package location
                    let pkg_location = std::env::current_dir()
                        .unwrap_or_default()
                        .join("node_modules")
                        .join(name)
                        .to_string_lossy()
                        .to_string();
                    pkg_info["location"] = json!(pkg_location);

                    // Check compatibility
                    if let Ok(compatible) =
                        check_package_compatibility(name, &installed_ver, &config.runtime.node)
                    {
                        pkg_info["compatible"] = json!(compatible);
                    }
                }
            }
        } else {
            pkg_info["version_installed"] = serde_json::Value::Null;
        }

        pkg_list.push(pkg_info);
    }

    let packages_info = json!({
        "declared": config.packages.len(),
        "installed": installed_count,
        "list": pkg_list
    });

    let mut status = json!({
        "project_root": cwd.to_string_lossy(),
        "config_path": toml_path.to_string_lossy(),
        "runtime": runtime_info,
        "packages": packages_info
    });

    // Add lock file info in verbose mode
    if verbose {
        let lock_file = cwd.join("ven.lock");
        status["lock_file"] = json!({
            "exists": lock_file.exists(),
            "path": lock_file.to_string_lossy()
        });
    }

    // Add env vars if present
    if !config.env.is_empty() {
        let mut env_list = Vec::new();
        for (key, value) in &config.env {
            let current = std::env::var(key).ok();
            env_list.push(json!({
                "key": key,
                "required": value,
                "active": current.as_deref() == Some(value.as_str())
            }));
        }
        status["environment"] = json!(env_list);
    }

    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}
