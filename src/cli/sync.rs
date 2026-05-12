//! `ven sync` — validate `ven.lock` graph, then install pinned versions.

use crate::cli::add::update_ven_toml_packages;
use crate::core::load_config;
use crate::core::packages;
use crate::core::requirements::{requirement_from_spec, Requirements};
use crate::intelligence::conflicts::analyze_npm_graph;
use crate::intelligence::engine::DependencyIntelligenceService;
use crate::intelligence::store::{IntelligenceStore, PACKAGE_CACHE_TTL_SECS};
use crate::intelligence::ven_lock::{
    compute_lock_content_hash, lock_to_intel_graph, validate_lock_graph, VenLockFile,
};
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn cmd_sync(dry_run: bool, json: bool, skip_validate: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    // Python-mode sync: if ven.toml declares Python runtime and no Node, sync
    // via requirements.txt + pip. Reconciles into [packages] so the source of
    // truth stays in ven.toml.
    let python_mode =
        !cfg.runtime.python.is_empty() && cfg.runtime.node.is_empty() && cfg.runtime.bun.is_empty();
    if python_mode {
        return sync_python(&cwd, dry_run, json);
    }

    let lock_path = cwd.join("ven.lock");
    if !lock_path.is_file() {
        anyhow::bail!(
            "No ven.lock in this directory.\n  Run: {} to generate a lockfile from ven.toml.",
            "ven lock".bold()
        );
    }

    if json {
        return sync_json(&lock_path, dry_run, skip_validate, &cfg, &cwd);
    }

    println!("Reading {}...", lock_path.display().to_string().cyan());
    let lock = VenLockFile::read_path(&lock_path).context("Failed to load ven.lock")?;

    if !skip_validate {
        println!("{}", "Validating dependency graph...".cyan());
        if let Err(e) = validate_lock_graph(&lock) {
            println!("  {} {}", "[ERROR]".red(), e);
            anyhow::bail!("Lock validation failed");
        }
        let graph_hash = compute_lock_content_hash(&lock)?;
        let lock_hash = lock
            .content_hash
            .clone()
            .unwrap_or_else(|| graph_hash.clone());

        let store = IntelligenceStore::open()?;
        store.upsert_packages_from_lock(&lock)?;
        store.upsert_dependencies_from_lock(&lock)?;
        let key = DependencyIntelligenceService::project_key(&cwd);
        store.record_lock_validation(&key, &graph_hash, &lock_hash)?;

        let graph = lock_to_intel_graph(&lock);
        let (peer_chains, _) = analyze_npm_graph(&graph, &cfg.packages);
        println!(
            "  {} Graph consistent ({} packages, {} peer/pin warnings)",
            "✓".green(),
            lock.packages.len(),
            peer_chains.len()
        );
        let _ = PACKAGE_CACHE_TTL_SECS;
    } else {
        println!(
            "  {} {}",
            "[WARN]".yellow(),
            "Skipped validation (--skip-validate)."
        );
    }

    if dry_run {
        println!("  {} Would install {} root package(s).", "[DRY-RUN]".yellow(), lock.roots.len());
        return Ok(());
    }

    println!("{}", "→ Installing...".cyan());
    for root in &lock.roots {
        let ver = lock
            .packages
            .get(root)
            .map(|p| p.version.as_str())
            .with_context(|| format!("root `{}` missing from lock packages", root))?;
        println!("  {} {}@{}", "[PKG]".cyan(), root.bold(), ver);
        packages::npm_install(root, ver)?;
    }

    println!("  {} Sync complete.", "[OK]".green());
    Ok(())
}

fn sync_json(
    lock_path: &Path,
    dry_run: bool,
    skip_validate: bool,
    cfg: &crate::core::config::VenConfig,
    cwd: &std::path::Path,
) -> Result<()> {
    let lock = VenLockFile::read_path(lock_path)?;
    let mut ok = true;
    let mut err_msg = None;
    if !skip_validate {
        if let Err(e) = validate_lock_graph(&lock) {
            ok = false;
            err_msg = Some(e.to_string());
        }
    }
    let out = serde_json::json!({
        "lock_path": lock_path.to_string_lossy(),
        "valid": ok,
        "error": err_msg,
        "package_count": lock.packages.len(),
        "edge_count": lock.edges.len(),
        "roots": lock.roots,
        "dry_run": dry_run,
        "project": cwd.to_string_lossy(),
        "runtime_node": cfg.runtime.node,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    if !ok {
        anyhow::bail!("validation failed");
    }
    if !dry_run && ok {
        for root in &lock.roots {
            if let Some(p) = lock.packages.get(root) {
                packages::npm_install(root, &p.version)?;
            }
        }
    }
    Ok(())
}

fn sync_python(cwd: &Path, dry_run: bool, json: bool) -> Result<()> {
    let req = Requirements::load_or_empty(cwd)?;
    let pinned = req.pinned();
    if json {
        let out = serde_json::json!({
            "mode": "python",
            "requirements_path": req.path().to_string_lossy(),
            "exists": req.exists(),
            "pinned_count": pinned.len(),
            "pinned": pinned,
            "dry_run": dry_run
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        if !req.exists() {
            return Ok(());
        }
    } else {
        println!(
            "{}",
            "Python sync via requirements.txt".bold().cyan()
        );
        println!("  {} {}", "Path:".dimmed(), req.path().display());
        if !req.exists() {
            println!(
                "  {} {} not present; nothing to sync.",
                "[INFO]".cyan(),
                "requirements.txt".bold()
            );
            return Ok(());
        }
        println!("  {} {} pinned entries", "[OK]".green(), pinned.len());
    }

    if dry_run {
        if !json {
            println!(
                "  {} Would run: pip install -r requirements.txt",
                "[DRY-RUN]".yellow()
            );
        }
        return Ok(());
    }

    let python = resolve_python_cmd();
    let status = Command::new(&python)
        .args(["-m", "pip", "install", "-r"])
        .arg(req.path())
        .status()
        .with_context(|| "Failed to invoke pip install -r requirements.txt")?;
    if !status.success() {
        anyhow::bail!("pip install -r requirements.txt failed (exit {:?})", status.code());
    }
    if !json {
        println!("  {} pip install completed", "[OK]".green());
    }

    let entries: Vec<(String, String)> = pinned
        .into_iter()
        .map(|(_, raw)| {
            let (name, raw_pin) = requirement_from_spec(&raw);
            (name, raw_pin)
        })
        .collect();
    if !entries.is_empty() {
        update_ven_toml_packages(&entries)?;
    }

    if !json {
        println!("  {} Sync complete.", "[OK]".green());
    }
    Ok(())
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
