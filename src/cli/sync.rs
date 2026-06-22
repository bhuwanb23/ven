//! `ven sync` — validate `ven.lock` graph, then install pinned versions.
//! `--check` adds drift detection (lock vs `node_modules` / installed pip).

use crate::cli::add::update_ven_toml_packages;
use crate::core::load_config;
use crate::core::packages;
use crate::core::requirements::{requirement_from_spec, Requirements};
use crate::intelligence::conflicts::analyze_npm_graph;
use crate::intelligence::drift::{compute_npm_drift, compute_python_drift, DriftReport};
use crate::intelligence::engine::DependencyIntelligenceService;
use crate::intelligence::store::{IntelligenceStore, PACKAGE_CACHE_TTL_SECS};
use crate::intelligence::ven_lock::{
    compute_lock_content_hash, lock_needs_upgrade, lock_to_intel_graph, validate_lock_graph,
    VenLockFile,
};
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn cmd_sync(dry_run: bool, check: bool, json: bool, skip_validate: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    // Python-mode sync: if ven.toml declares Python runtime and no Node, sync
    // via requirements.txt + pip. Reconciles into [packages] so the source of
    // truth stays in ven.toml.
    let python_mode =
        !cfg.runtime.python.is_empty() && cfg.runtime.node.is_empty() && cfg.runtime.bun.is_empty();
    if python_mode {
        return sync_python(&cwd, dry_run, check, json, &cfg);
    }

    // PHP-mode: `ven sync` is not supported (no lockfile equivalent).
    let php_mode =
        !cfg.runtime.php.is_empty() && cfg.runtime.node.is_empty() && cfg.runtime.bun.is_empty() && cfg.runtime.python.is_empty();
    if php_mode {
        println!(
            "{} `ven sync` is only supported for npm and Python-based projects.",
            "[INFO]".cyan()
        );
        return Ok(());
    }

    let lock_path = cwd.join("ven.lock");
    if !lock_path.is_file() {
        anyhow::bail!(
            "No ven.lock in this directory.\n  Run: {} to generate a lockfile from ven.toml.",
            "ven lock".bold()
        );
    }

    if json {
        return sync_json(&lock_path, dry_run, check, skip_validate, &cfg, &cwd);
    }

    println!("Reading {}...", lock_path.display().to_string().cyan());
    let lock = VenLockFile::read_path(&lock_path).context("Failed to load ven.lock")?;

    if lock_needs_upgrade(&lock) {
        println!(
            "  {} {}",
            "[INFO]".cyan(),
            format!(
                "ven.lock is at format v{} — regenerate with `ven lock` to gain SRI integrity hashes.",
                lock.lock_format_version
            )
            .dimmed()
        );
    }

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

    if check {
        let report = compute_npm_drift(&cwd, &lock, &cfg)?;
        print_drift_report_npm(&cwd, &report);
        if report.has_drift() {
            anyhow::bail!("ven sync --check: drift detected");
        }
        return Ok(());
    }

    if dry_run {
        println!(
            "  {} Would install {} root package(s).",
            "[DRY-RUN]".yellow(),
            lock.roots.len()
        );
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
    check: bool,
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

    let drift_value = if ok && check {
        let r = compute_npm_drift(cwd, &lock, cfg)?;
        Some(serde_json::to_value(&r)?)
    } else {
        None
    };
    let drift_actionable = drift_value
        .as_ref()
        .and_then(|v| v.get("missing").and_then(|x| x.as_array()).map(|a| a.len()))
        .unwrap_or(0)
        + drift_value
            .as_ref()
            .and_then(|v| v.get("stale").and_then(|x| x.as_array()).map(|a| a.len()))
            .unwrap_or(0)
        + drift_value
            .as_ref()
            .and_then(|v| {
                v.get("missing_from_lock")
                    .and_then(|x| x.as_array())
                    .map(|a| a.len())
            })
            .unwrap_or(0)
        + drift_value
            .as_ref()
            .and_then(|v| {
                v.get("config_mismatches")
                    .and_then(|x| x.as_array())
                    .map(|a| a.len())
            })
            .unwrap_or(0);

    let out = serde_json::json!({
        "lock_path": lock_path.to_string_lossy(),
        "lock_format_version": lock.lock_format_version,
        "needs_upgrade": lock_needs_upgrade(&lock),
        "valid": ok,
        "error": err_msg,
        "package_count": lock.packages.len(),
        "edge_count": lock.edges.len(),
        "roots": lock.roots,
        "dry_run": dry_run,
        "check": check,
        "drift": drift_value,
        "project": cwd.to_string_lossy(),
        "runtime_node": cfg.runtime.node,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    if !ok {
        anyhow::bail!("validation failed");
    }
    if check && drift_actionable > 0 {
        anyhow::bail!("ven sync --check: drift detected");
    }
    if !dry_run && !check && ok {
        for root in &lock.roots {
            if let Some(p) = lock.packages.get(root) {
                packages::npm_install(root, &p.version)?;
            }
        }
    }
    Ok(())
}

fn sync_python(
    cwd: &Path,
    dry_run: bool,
    check: bool,
    json: bool,
    cfg: &crate::core::config::VenConfig,
) -> Result<()> {
    let req = Requirements::load_or_empty(cwd)?;
    let pinned = req.pinned();

    if check {
        let report = compute_python_drift(cwd, cfg, &pinned)?;
        if json {
            let out = serde_json::json!({
                "mode": "python",
                "check": true,
                "requirements_path": req.path().to_string_lossy(),
                "exists": req.exists(),
                "drift": report,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            print_drift_report_python(cwd, &report);
        }
        if report.has_drift() {
            anyhow::bail!("ven sync --check: drift detected");
        }
        return Ok(());
    }

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
        println!("{}", "Python sync via requirements.txt".bold().cyan());
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
        anyhow::bail!(
            "pip install -r requirements.txt failed (exit {:?})",
            status.code()
        );
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

fn print_drift_report_npm(cwd: &Path, report: &DriftReport) {
    println!();
    println!(
        "{} {}",
        "Drift report".bold(),
        format!("({})", cwd.display()).dimmed()
    );
    if !report.missing.is_empty() {
        println!(
            "  {} {} package(s) in ven.lock are not installed in node_modules/",
            "[MISSING]".red(),
            report.missing.len()
        );
        for name in &report.missing {
            println!("    - {}", name);
        }
    }
    if !report.stale.is_empty() {
        println!(
            "  {} {} package(s) installed at the wrong version",
            "[STALE]".yellow(),
            report.stale.len()
        );
        for s in &report.stale {
            println!(
                "    - {}: lock={} installed={}",
                s.package, s.locked, s.installed
            );
        }
    }
    if !report.missing_from_lock.is_empty() {
        println!(
            "  {} {} root(s) in ven.toml [packages] missing from ven.lock — run `ven lock`",
            "[OUT-OF-LOCK]".yellow(),
            report.missing_from_lock.len()
        );
        for name in &report.missing_from_lock {
            println!("    - {}", name);
        }
    }
    if !report.config_mismatches.is_empty() {
        println!(
            "  {} {} root(s) whose ven.toml constraint is not satisfied by ven.lock",
            "[MISMATCH]".yellow(),
            report.config_mismatches.len()
        );
        for m in &report.config_mismatches {
            println!(
                "    - {}: ven.toml=`{}` lock={}",
                m.package, m.ven_toml_spec, m.lock_pin
            );
        }
    }
    if !report.orphan.is_empty() {
        println!(
            "  {} {} package(s) in node_modules/ but not in ven.lock (transitive — informational)",
            "[ORPHAN]".dimmed(),
            report.orphan.len()
        );
    }
    if report.has_drift() {
        println!(
            "  {} {} actionable issue(s).",
            "[FAIL]".red().bold(),
            report.count_actionable()
        );
    } else {
        println!(
            "  {} No drift — lock and node_modules agree.",
            "[OK]".green()
        );
    }
}

fn print_drift_report_python(cwd: &Path, report: &DriftReport) {
    println!();
    println!(
        "{} {}",
        "Drift report (python)".bold(),
        format!("({})", cwd.display()).dimmed()
    );
    if !report.missing.is_empty() {
        println!(
            "  {} {} package(s) declared but not installed (`pip list`)",
            "[MISSING]".red(),
            report.missing.len()
        );
        for name in &report.missing {
            println!("    - {}", name);
        }
    }
    if !report.stale.is_empty() {
        println!(
            "  {} {} package(s) installed but constraint not satisfied",
            "[STALE]".yellow(),
            report.stale.len()
        );
        for s in &report.stale {
            println!(
                "    - {}: declared=`{}` installed={}",
                s.package, s.locked, s.installed
            );
        }
    }
    if report.has_drift() {
        println!(
            "  {} {} actionable issue(s).",
            "[FAIL]".red().bold(),
            report.count_actionable()
        );
    } else {
        println!("  {} No drift.", "[OK]".green());
    }
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
            let p = crate::core::ven_home::ven_home()
                .join("python")
                .join(ver)
                .join("python.exe");
            if p.is_file() {
                return p;
            }
        }
        PathBuf::from("python")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("python3")
    }
}
