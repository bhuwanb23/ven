mod languages;
mod toml;

use crate::core::packages;
use crate::core::{load_config, DependencyGraph, SecurityScanner};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use languages::{
    cmd_add_deno_notice, cmd_add_go, cmd_add_java_notice, cmd_add_python, cmd_add_ruby,
    cmd_add_rust,
};
pub use toml::update_ven_toml_packages;

/// Load existing packages from ven.toml [packages] section.
fn load_existing_packages() -> Result<HashMap<String, String>> {
    let cwd = std::env::current_dir()?;
    match load_config(&cwd)? {
        Some(config) => Ok(config.packages),
        None => Ok(HashMap::new()),
    }
}

/// Package entry for batch processing.
struct PackageEntry {
    name: String,
    pinned_version: Option<String>,
}

/// Add packages with pre-flight dependency analysis.
pub fn cmd_add(
    package_specs: &[String],
    skip_check: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    if package_specs.is_empty() {
        println!("  {} No packages specified", "[ERROR]".red());
        println!(
            "  {} Usage: ven add <package> [package...] [--dry-run] [--verbose]",
            "[TIP]".cyan()
        );
        return Ok(());
    }

    let packages: Vec<PackageEntry> = package_specs
        .iter()
        .map(|spec| {
            let (name, pinned_version) = if spec.contains('@') && !spec.starts_with('@') {
                let parts: Vec<&str> = spec.splitn(2, '@').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (spec.clone(), None)
            };
            PackageEntry {
                name,
                pinned_version,
            }
        })
        .collect();

    let cwd = std::env::current_dir()?;
    let cfg =
        load_config(&cwd)?.ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;
    let node_version = cfg.runtime.node.clone();
    let python_version = cfg.runtime.python.clone();
    let go_version = cfg.runtime.go.clone();
    let rust_version = cfg.runtime.rust.clone();
    let java_version = cfg.runtime.java.clone();
    let deno_version = cfg.runtime.deno.clone();
    let ruby_version = cfg.runtime.ruby.clone();
    let python_mode = !python_version.is_empty() && node_version.is_empty();
    let go_mode = !go_version.is_empty()
        && node_version.is_empty()
        && python_version.is_empty()
        && ruby_version.is_empty();
    let rust_mode = !rust_version.is_empty()
        && node_version.is_empty()
        && python_version.is_empty()
        && go_version.is_empty()
        && ruby_version.is_empty();
    let java_mode = !java_version.is_empty()
        && node_version.is_empty()
        && python_version.is_empty()
        && go_version.is_empty()
        && rust_version.is_empty()
        && ruby_version.is_empty();
    let deno_mode = !deno_version.is_empty()
        && node_version.is_empty()
        && python_version.is_empty()
        && go_version.is_empty()
        && rust_version.is_empty()
        && java_version.is_empty()
        && ruby_version.is_empty();

    let ruby_mode = !ruby_version.is_empty()
        && node_version.is_empty()
        && python_version.is_empty()
        && go_version.is_empty()
        && rust_version.is_empty()
        && java_version.is_empty()
        && deno_version.is_empty();

    if python_mode {
        return cmd_add_python(package_specs, dry_run);
    }
    if go_mode {
        return cmd_add_go(package_specs, dry_run);
    }
    if rust_mode {
        return cmd_add_rust(package_specs, dry_run);
    }
    if java_mode {
        return cmd_add_java_notice(package_specs, dry_run);
    }
    if deno_mode {
        return cmd_add_deno_notice(package_specs, dry_run);
    }
    if ruby_mode {
        return cmd_add_ruby(package_specs, dry_run);
    }

    let existing_packages = load_existing_packages()?;
    if !existing_packages.is_empty() {
        println!(
            "  {} {} existing package(s) in ven.toml",
            "[EXISTING]".cyan(),
            existing_packages.len()
        );
    }

    println!("\n{}", "ven add".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), packages.len());
    println!("  {} runtime node {}", "[RUNTIME]".cyan(), node_version);

    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
    }
    println!();

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut all_graphs: Vec<(String, DependencyGraph)> = Vec::new();

    for pkg in &packages {
        println!("[INFO] Analyzing {}...", pkg.name.bold());
        let version_spec = if let Some(ref v) = pkg.pinned_version {
            v.clone()
        } else {
            "latest".to_string()
        };

        match analyze_package(
            &pkg.name,
            &version_spec,
            &node_version,
            &existing_packages,
            skip_check,
        ) {
            Ok(graph) => {
                if verbose {
                    println!();
                    println!("    {}", "Dependency Tree:".dimmed());
                    graph.print_tree();
                    graph.print_tree_summary();
                    graph.print_transitive_analysis();
                    println!();
                }

                if !graph.incompatibilities.is_empty() {
                    println!(
                        "  {} {} compatibility warning(s):",
                        "[WARN]".yellow(),
                        graph.incompatibilities.len()
                    );
                    for incompat in &graph.incompatibilities {
                        println!(
                            "    {} {} requires Node {}",
                            "[!]".yellow(),
                            incompat.package,
                            incompat.required_node
                        );
                    }
                }

                if !graph.conflicts.is_empty() {
                    println!(
                        "\n  {} {} version conflict(s) detected:",
                        "[WARN]".yellow().bold(),
                        graph.conflicts.len()
                    );
                    for conflict in &graph.conflicts {
                        println!("\n    {} {}", "[WARN]".yellow(), conflict.package.bold());
                        println!("      {} Required by:", "├".yellow());

                        for (i, (requirer, constraint)) in conflict.constraints.iter().enumerate() {
                            let connector = if i == conflict.constraints.len() - 1 {
                                "└"
                            } else {
                                "├"
                            };
                            println!(
                                "        {} {} ({})",
                                connector.yellow(),
                                requirer,
                                constraint
                            );
                        }

                        println!(
                            "      {} Versions: {}",
                            "├".yellow(),
                            conflict.versions.join(", ").bold()
                        );
                    }

                    graph.print_resolution_suggestions();
                    println!();
                }

                let has_critical = graph
                    .incompatibilities
                    .iter()
                    .any(|i| i.package == pkg.name);

                if has_critical && !skip_check {
                    println!(
                        "  {} {} is not compatible with node {}",
                        "[ERROR]".red(),
                        pkg.name,
                        node_version
                    );
                    fail_count += 1;
                    continue;
                }

                println!(
                    "  {} {} will install: {} total packages",
                    "[OK]".green(),
                    pkg.name,
                    graph.nodes.len()
                );

                all_graphs.push((pkg.name.clone(), graph));
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {}", "[ERROR]".red(), e.to_string().red());
                fail_count += 1;
            }
        }
    }

    println!();
    println!("  {}", "Analysis Summary".bold().cyan());
    println!(
        "    {} {} package(s) analyzed",
        "Total:".dimmed(),
        packages.len()
    );
    println!(
        "    {} {}",
        "Compatible:".dimmed(),
        success_count.to_string().green()
    );
    if fail_count > 0 {
        println!(
            "    {} {}",
            "Failed:".dimmed(),
            fail_count.to_string().red()
        );
    } else {
        println!("    {} {}", "Failed:".dimmed(), "0".green());
    }

    let total_packages: u32 = all_graphs.iter().map(|(_, g)| g.nodes.len() as u32).sum();
    println!(
        "    {} {} packages to install",
        "Total:".dimmed(),
        total_packages
    );

    if !all_graphs.is_empty() {
        println!("\n  {}", "Installation Preview".bold().cyan());

        for (pkg_name, graph) in &all_graphs {
            let pkg = packages.iter().find(|p| p.name == *pkg_name).unwrap();
            let resolved_version = graph
                .nodes
                .get(pkg_name)
                .map(|n| n.version.clone())
                .unwrap_or_else(|| "unknown".to_string());

            println!(
                "\n    {} {}@{}",
                "[PKG]".cyan(),
                pkg_name.bold(),
                resolved_version.bold()
            );

            let transitive_deps = graph.nodes.len().saturating_sub(1);
            println!(
                "      {} {} direct + {} transitive dependencies",
                "├".dimmed(),
                graph
                    .nodes
                    .get(pkg_name)
                    .map(|n| n.dependencies.len())
                    .unwrap_or(0),
                transitive_deps
            );

            let has_conflicts = !graph.conflicts.is_empty();
            let has_incompatibilities = !graph.incompatibilities.is_empty();

            if has_conflicts || has_incompatibilities {
                println!(
                    "      {} {}",
                    "├".dimmed(),
                    "[WARN] Has conflicts or incompatibilities".yellow()
                );
            } else {
                println!(
                    "      {} {}",
                    "├".dimmed(),
                    "[OK] No conflicts detected".green()
                );
            }

            let version_to_add = if let Some(ref v) = pkg.pinned_version {
                v.clone()
            } else {
                format!("^{}", resolved_version.split('.').next().unwrap_or("0"))
            };

            println!(
                "      {} {} {} = \"{}\"",
                "└".dimmed(),
                "📝".cyan(),
                pkg_name,
                version_to_add
            );
        }
        println!();
    }

    println!("\n  {}", "Security Audit".bold().cyan());
    let mut all_packages: HashMap<String, String> = HashMap::new();
    for (_, graph) in &all_graphs {
        for (name, node) in &graph.nodes {
            all_packages.insert(name.clone(), node.version.clone());
        }
    }

    println!(
        "  {} Scanning {} packages for known vulnerabilities...",
        "🔒".cyan(),
        all_packages.len()
    );

    let vulnerabilities = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| {
            handle.block_on(async {
                let scanner = SecurityScanner::new()?;
                scanner.scan_packages(&all_packages).await
            })
        })
    } else {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let scanner = SecurityScanner::new()?;
            scanner.scan_packages(&all_packages).await
        })
    };

    let vulnerabilities = match vulnerabilities {
        Ok(advisories) => advisories,
        Err(e) => {
            eprintln!("  {} Warning: Security scan failed: {}", "⚠".yellow(), e);
            Vec::new()
        }
    };

    let scanner = SecurityScanner::new()?;
    scanner.print_audit(&vulnerabilities);

    if scanner.has_critical_vulnerabilities(&vulnerabilities) {
        println!(
            "\n  {} Critical/High vulnerabilities detected!",
            "🚨".red().bold()
        );
        println!("  {} Consider updating to patched versions", "⚠".yellow());
    }

    if dry_run {
        println!();
        println!(
            "  {} Dry run complete - no packages were installed",
            "[DRY-RUN]".yellow()
        );
        println!("  {} Run without --dry-run to install", "[TIP]".cyan());
        println!();
        return Ok(());
    }

    if fail_count > 0 {
        println!();
        println!(
            "  {} Cannot proceed - fix errors before installing",
            "[ERROR]".red()
        );
        println!(
            "  {} Use --skip-check to bypass compatibility checks",
            "[TIP]".cyan()
        );
        println!();
        return Ok(());
    }

    println!();
    println!(
        "  {} Ready to install {} package(s)? (y/N)",
        "[INSTALL]".green().bold(),
        success_count
    );

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("  {} Installation cancelled", "[CANCELLED]".yellow());
        println!();
        return Ok(());
    }

    println!();
    let mut installed = Vec::new();

    for (pkg_name, graph) in &all_graphs {
        let pkg = packages.iter().find(|p| p.name == *pkg_name).unwrap();
        println!("[INFO] Installing {}...", pkg_name.bold());

        let version_to_install = graph
            .nodes
            .get(pkg_name)
            .map(|node| node.version.clone())
            .unwrap_or_else(|| {
                if let Some(ref v) = pkg.pinned_version {
                    v.clone()
                } else {
                    "latest".to_string()
                }
            });

        match packages::npm_install(&pkg.name, &version_to_install) {
            Ok(()) => {
                println!(
                    "  {} {}@{} installed",
                    "[OK]".green(),
                    pkg.name,
                    version_to_install
                );
                installed.push((pkg.name.clone(), version_to_install));
            }
            Err(e) => {
                println!(
                    "  {} Failed to install {}: {}",
                    "[ERROR]".red(),
                    pkg.name,
                    e
                );
            }
        }
    }

    if !installed.is_empty() {
        println!();
        update_ven_toml_packages(&installed)?;
    }

    println!();
    Ok(())
}

/// Analyze a package and build dependency graph.
fn analyze_package(
    pkg_name: &str,
    version_spec: &str,
    node_version: &str,
    existing_packages: &HashMap<String, String>,
    _skip_check: bool,
) -> Result<DependencyGraph> {
    use tokio::runtime::Runtime;

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| {
            handle.block_on(async {
                let mut graph = DependencyGraph::new(node_version.to_string());
                graph
                    .build(pkg_name, version_spec, existing_packages)
                    .await?;
                Ok(graph)
            })
        })
    } else {
        let rt = Runtime::new()?;
        rt.block_on(async {
            let mut graph = DependencyGraph::new(node_version.to_string());
            graph
                .build(pkg_name, version_spec, existing_packages)
                .await?;
            Ok(graph)
        })
    }
}
