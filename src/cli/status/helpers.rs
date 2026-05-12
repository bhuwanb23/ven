use crate::core::config::VenConfig;
use crate::core::{
    resolve_bun_version, resolve_deno_version, resolve_go_version, resolve_java_version,
    resolve_node_version, resolve_python_version, resolve_ruby_version, resolve_rust_version,
};
use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn resolve_version_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;

    let registry = PluginRegistry::new();
    let plugin = registry.require("node")?;
    let installed = plugin.list_installed().unwrap_or_default();

    match resolve_node_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_python_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("python")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_python_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_go_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("go")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_go_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_rust_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("rust")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_rust_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_java_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("java")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_java_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_deno_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("deno")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_deno_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_bun_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("bun")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_bun_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn resolve_ruby_for_display(spec: &str) -> Result<String> {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    let plugin = registry.require("ruby")?;
    let installed = plugin.list_installed().unwrap_or_default();
    match resolve_ruby_version(spec, &installed) {
        Ok(resolved) => Ok(resolved),
        Err(_) => Ok(spec.to_string()),
    }
}

pub(super) fn is_version_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;

    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("node") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_node_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_python_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("python") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_python_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_go_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("go") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_go_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_rust_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("rust") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_rust_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_java_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("java") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_java_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_deno_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("deno") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_deno_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_ruby_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("ruby") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_ruby_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn is_bun_installed(spec: &str) -> bool {
    use crate::plugins::PluginRegistry;
    let registry = PluginRegistry::new();
    if let Ok(plugin) = registry.require("bun") {
        let installed = plugin.list_installed().unwrap_or_default();
        resolve_bun_version(spec, &installed).is_ok()
    } else {
        false
    }
}

pub(super) fn get_bin_path_for_version(spec: &str) -> Result<PathBuf> {
    use crate::plugins::PluginRegistry;

    let registry = PluginRegistry::new();
    let plugin = registry.require("node")?;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = resolve_node_version(spec, &installed)?;

    plugin.bin_path(&resolved)
}

pub(super) fn is_package_installed(package: &str) -> bool {
    let pkg_json = std::env::current_dir()
        .unwrap_or_default()
        .join("node_modules")
        .join(package)
        .join("package.json");

    pkg_json.exists()
}

pub(super) fn is_python_package_installed(package: &str) -> bool {
    let out = Command::new(resolve_python_cmd())
        .args(["-m", "pip", "show", package])
        .output();
    matches!(out, Ok(o) if o.status.success())
}

fn resolve_python_cmd() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
            let p = PathBuf::from(venv).join("Scripts").join("python.exe");
            if p.is_file() {
                return p;
            }
        }
        if let Ok(ver) = std::env::var("VEN_PYTHON_VERSION") {
            if let Some(home) = dirs::home_dir() {
                let p = home
                    .join(".ven")
                    .join("python")
                    .join(ver)
                    .join("python.exe");
                if p.is_file() {
                    return p;
                }
            }
        }
        PathBuf::from("python")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("python3")
    }
}

pub(super) fn get_installed_package_version(package: &str) -> Result<String> {
    crate::core::packages::get_installed_version(package)
}

pub(super) fn check_package_compatibility(
    pkg_name: &str,
    _pkg_version: &str,
    node_spec: &str,
) -> Result<bool> {
    use crate::intelligence::engine::DependencyIntelligenceService;

    Ok(DependencyIntelligenceService::npm_latest_compatible(pkg_name, node_spec)?.is_some())
}

pub(super) fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                total_size += entry.metadata()?.len();
            } else if path.is_dir() {
                total_size += calculate_dir_size(&path)?;
            }
        }
    }

    Ok(total_size)
}

pub(super) fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub(super) fn check_if_version_active(spec: &str) -> Result<bool> {
    let current_path = std::env::var("PATH").unwrap_or_default();
    let bin_path = get_bin_path_for_version(spec)?;

    Ok(current_path.contains(&bin_path.parent().unwrap().to_string_lossy().to_string()))
}

pub(super) fn print_health_summary(config: &VenConfig) -> Result<()> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut ok_items = Vec::new();

    // Check runtimes
    if config.runtime.node.is_empty()
        && config.runtime.python.is_empty()
        && config.runtime.go.is_empty()
        && config.runtime.rust.is_empty()
        && config.runtime.java.is_empty()
        && config.runtime.deno.is_empty()
        && config.runtime.bun.is_empty()
        && config.runtime.ruby.is_empty()
    {
        issues.push("No runtime version specified".to_string());
    }
    if !config.runtime.node.is_empty() {
        if !is_version_installed(&config.runtime.node) {
            issues.push(format!("Node {} not installed", config.runtime.node));
        } else {
            ok_items.push(format!("Node {} ready", config.runtime.node));
        }
    }
    if !config.runtime.python.is_empty() {
        if !is_python_installed(&config.runtime.python) {
            issues.push(format!("Python {} not installed", config.runtime.python));
        } else {
            ok_items.push(format!("Python {} ready", config.runtime.python));
        }
    }
    if !config.runtime.go.is_empty() {
        if !is_go_installed(&config.runtime.go) {
            issues.push(format!("Go {} not installed", config.runtime.go));
        } else {
            ok_items.push(format!("Go {} ready", config.runtime.go));
        }
    }
    if !config.runtime.rust.is_empty() {
        if !is_rust_installed(&config.runtime.rust) {
            issues.push(format!("Rust {} not installed", config.runtime.rust));
        } else {
            ok_items.push(format!("Rust {} ready", config.runtime.rust));
        }
    }
    if !config.runtime.java.is_empty() {
        if !is_java_installed(&config.runtime.java) {
            issues.push(format!("Java {} not installed", config.runtime.java));
        } else {
            ok_items.push(format!("Java {} ready", config.runtime.java));
        }
    }
    if !config.runtime.deno.is_empty() {
        if !is_deno_installed(&config.runtime.deno) {
            issues.push(format!("Deno {} not installed", config.runtime.deno));
        } else {
            ok_items.push(format!("Deno {} ready", config.runtime.deno));
        }
    }
    if !config.runtime.bun.is_empty() {
        if !is_bun_installed(&config.runtime.bun) {
            issues.push(format!("Bun {} not installed", config.runtime.bun));
        } else {
            ok_items.push(format!("Bun {} ready", config.runtime.bun));
        }
    }
    if !config.runtime.ruby.is_empty() {
        if !is_ruby_installed(&config.runtime.ruby) {
            issues.push(format!("Ruby {} not installed", config.runtime.ruby));
        } else {
            ok_items.push(format!("Ruby {} ready", config.runtime.ruby));
        }
    }

    // Check packages
    let pkg_count = config.packages.len();
    if pkg_count > 0 {
        let mut missing = 0;
        for pkg_name in config.packages.keys() {
            if !is_package_installed(pkg_name) {
                missing += 1;
            }
        }

        if missing == 0 {
            ok_items.push(format!("All {} packages installed", pkg_count));
        } else {
            warnings.push(format!("{}/{} packages missing", missing, pkg_count));
        }
    }

    // Print summary
    if !issues.is_empty() {
        for issue in &issues {
            println!("    {} {}", "✗".red(), issue);
        }
    }

    if !warnings.is_empty() {
        for warning in &warnings {
            println!("    {} {}", "⚠".yellow(), warning);
        }
    }

    for ok in &ok_items {
        println!("    {} {}", "✓".green(), ok);
    }

    if issues.is_empty() && warnings.is_empty() {
        println!("    {} {}", "✓".green(), "All checks passed!".green());
    }

    if !config.runtime.node.is_empty() || !config.runtime.bun.is_empty() {
        if let Ok(cwd) = std::env::current_dir() {
            let key = crate::intelligence::engine::DependencyIntelligenceService::project_key(&cwd);
            if let Ok(Some(snap)) =
                crate::intelligence::engine::DependencyIntelligenceService::load_snapshot(&key)
            {
                if snap.compatible {
                    println!(
                        "    {} {}",
                        "✓".green(),
                        "Dependency intelligence snapshot: compatible".green()
                    );
                } else {
                    println!(
                        "    {} {}",
                        "⚠".yellow(),
                        "Dependency intelligence snapshot: conflicts — try `ven check-add` or `ven graph`"
                            .yellow()
                    );
                }
            }
        }
    }

    Ok(())
}

pub(super) fn auto_install_version(language: &str, spec: &str) -> Result<()> {
    println!(
        "\n  {} Installing {} {}...",
        "[AUTO-FIX]".cyan(),
        language,
        spec
    );
    crate::cli::install::cmd_install(language, spec)?;
    println!("  {} {} {} installed", "✓".green(), language, spec);
    Ok(())
}

pub(super) fn auto_install_package(pkg_name: &str, pkg_version: &str) -> Result<()> {
    println!(
        "  {} Installing {}@{}...",
        "[AUTO-FIX]".cyan(),
        pkg_name,
        pkg_version
    );

    let packages = vec![format!("{}@{}", pkg_name, pkg_version)];
    crate::cli::add::cmd_add(&packages, false, false, false)?;

    println!("  {} {}@{} installed", "✓".green(), pkg_name, pkg_version);
    Ok(())
}
