//! `ven lock` — write `ven.lock` from merged dependency intelligence graphs.

use crate::core::load_config;
use crate::intelligence::engine::DependencyIntelligenceService;
use crate::intelligence::graph::RuntimeKind;
use crate::intelligence::ven_lock::{validate_lock_graph, VenLockFile};
use anyhow::Result;
use colored::Colorize;

pub fn cmd_lock() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    if cfg.runtime.node.is_empty() && cfg.runtime.bun.is_empty() {
        println!(
            "{} `ven lock` is only supported for npm-based projects (set [runtime] node or bun).",
            "[INFO]".cyan()
        );
        return Ok(());
    }

    if cfg.packages.is_empty() {
        println!(
            "{} No packages in ven.toml — add packages with `ven add` before locking.",
            "[INFO]".cyan()
        );
        return Ok(());
    }

    println!("{}", "ven lock".bold().cyan());
    println!("  {} Resolving dependency graphs…", "[INFO]".cyan());

    let mut keys: Vec<String> = cfg.packages.keys().cloned().collect();
    keys.sort();

    let mut graphs = Vec::new();
    for name in &keys {
        let spec = cfg
            .packages
            .get(name)
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("latest");
        let sim = DependencyIntelligenceService::simulate_add(&cfg, name, spec, &cfg.packages)?;
        graphs.push(sim.graph);
    }

    let (runtime_kind, runtime_version) = if !cfg.runtime.node.is_empty() {
        (RuntimeKind::NpmFamily, cfg.runtime.node.clone())
    } else {
        (RuntimeKind::NpmFamily, cfg.runtime.bun.clone())
    };

    let lock = VenLockFile::from_merged_simulations(runtime_kind, runtime_version, &keys, &graphs)?;
    validate_lock_graph(&lock)?;

    let path = cwd.join("ven.lock");
    lock.write_path(&path)?;

    let with_integrity = lock
        .packages
        .values()
        .filter(|p| p.integrity.is_some())
        .count();
    let total = lock.packages.len();

    println!(
        "  {} Wrote {} ({} packages, {} edges, hash {})",
        "[OK]".green(),
        path.display(),
        total,
        lock.edges.len(),
        lock.content_hash.as_deref().unwrap_or("?")
    );
    println!(
        "  {} Integrity: {}/{} packages have SRI hashes (npm `dist.integrity`)",
        "[INFO]".cyan(),
        with_integrity,
        total
    );
    Ok(())
}
