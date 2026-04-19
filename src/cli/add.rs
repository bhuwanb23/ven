use anyhow::Result;
use colored::Colorize;
use toml_edit::{DocumentMut, value};
use crate::core::{load_config, packages::*, DependencyGraph};

/// Package entry for batch processing
struct PackageEntry {
    name: String,
    pinned_version: Option<String>,
}

/// Add packages with pre-flight dependency analysis
pub fn cmd_add(package_specs: &[String], skip_check: bool, dry_run: bool, verbose: bool) -> Result<()> {
    if package_specs.is_empty() {
        println!("  {} No packages specified", "[ERROR]".red());
        println!("  {} Usage: ven add <package> [package...] [--dry-run] [--verbose]", "[TIP]".cyan());
        return Ok(());
    }

    // Parse all package specs
    let packages: Vec<PackageEntry> = package_specs
        .iter()
        .map(|spec| {
            let (name, pinned_version) = if spec.contains('@') && !spec.starts_with('@') {
                let parts: Vec<&str> = spec.splitn(2, '@').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (spec.clone(), None)
            };
            PackageEntry { name, pinned_version }
        })
        .collect();

    // Get current Node version
    let cwd = std::env::current_dir()?;
    let node_version = load_config(&cwd)?
        .map(|c| c.runtime.node)
        .unwrap_or_else(|| "0".to_string());

    println!("\n{}", "ven add".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), packages.len());
    println!("  {} Node.js {}", "[RUNTIME]".cyan(), node_version);
    
    if dry_run {
        println!("  {} Dry run mode - no changes will be made", "[DRY-RUN]".yellow());
    }
    println!();

    // Phase 1: Pre-flight analysis for each package
    let mut success_count = 0;
    let mut fail_count = 0;
    let mut all_graphs: Vec<(String, DependencyGraph)> = Vec::new();

    for pkg in &packages {
        println!("{} Analyzing {}...", "→".cyan(), pkg.name.bold());

        // Build dependency graph with pre-flight analysis
        let version_spec = if let Some(ref v) = pkg.pinned_version {
            v.clone()
        } else {
            "latest".to_string()
        };

        match analyze_package(&pkg.name, &version_spec, &node_version, skip_check) {
            Ok(mut graph) => {
                // Print dependency tree
                if verbose {
                    println!();
                    println!("    {}", "Dependency Tree:".dimmed());
                    graph.print_tree();
                    println!();
                }

                // Show warnings
                if !graph.incompatibilities.is_empty() {
                    println!("  {} {} compatibility warning(s):", "[WARN]".yellow(), graph.incompatibilities.len());
                    for incompat in &graph.incompatibilities {
                        println!("    {} {} requires Node {}", 
                            "[!]".yellow(),
                            incompat.package,
                            incompat.required
                        );
                    }
                }

                if !graph.conflicts.is_empty() {
                    println!("  {} {} version conflict(s) detected:", "[WARN]".yellow(), graph.conflicts.len());
                    for conflict in &graph.conflicts {
                        println!("    {} {} needed by {} and {}", 
                            "[!]".yellow(),
                            conflict.package,
                            conflict.requirement1,
                            conflict.requirement2
                        );
                    }
                }

                // Check for critical errors
                let has_critical = graph.incompatibilities.iter()
                    .any(|i| i.package == pkg.name && i.depth == 0);
                
                if has_critical && !skip_check {
                    println!("  {} {} is not compatible with Node.js {}", 
                        "[ERROR]".red(), pkg.name, node_version);
                    fail_count += 1;
                    continue;
                }

                let stats = graph.install_preview();
                println!("  {} {} will install: {} total packages ({:.2} KB)",
                    "[OK]".green(),
                    pkg.name,
                    stats.total_packages,
                    stats.total_size_kb
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

    // Phase 2: Summary
    println!();
    println!("  {}", "Analysis Summary".bold().cyan());
    println!("    {} {} package(s) analyzed", "Total:".dimmed(), packages.len());
    println!("    {} {}", "Compatible:".dimmed(), success_count.to_string().green());
    
    if fail_count > 0 {
        println!("    {} {}", "Failed:".dimmed(), fail_count.to_string().red());
    } else {
        println!("    {} {}", "Failed:".dimmed(), "0".green());
    }

    // Phase 3: Calculate totals
    let total_packages: u32 = all_graphs.iter()
        .map(|(_, g)| g.nodes.len() as u32)
        .sum();
    let total_size: f64 = all_graphs.iter()
        .map(|(_, g)| g.install_preview().total_size_kb)
        .sum();

    println!("    {} {} packages to install", "Total:".dimmed(), total_packages);
    println!("    {} {:.2} KB download size", "Size:".dimmed(), total_size);

    // Phase 4: Dry run or install
    if dry_run {
        println!();
        println!("  {} Dry run complete - no packages were installed", "[DRY-RUN]".yellow());
        println!("  {} Run without --dry-run to install", "[TIP]".cyan());
        println!();
        return Ok(());
    }

    if fail_count > 0 {
        println!();
        println!("  {} Cannot proceed - fix errors before installing", "[ERROR]".red());
        println!("  {} Use --skip-check to bypass compatibility checks", "[TIP]".cyan());
        println!();
        return Ok(());
    }

    // Phase 5: User confirmation
    println!();
    println!("  {} Ready to install {} package(s)? (y/N)", "[INSTALL]".green().bold(), success_count);
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    if input != "y" && input != "yes" {
        println!("  {} Installation cancelled", "[CANCELLED]".yellow());
        println!();
        return Ok(());
    }

    // Phase 6: Install packages
    println!();
    let mut installed = Vec::new();

    for (pkg_name, _) in &all_graphs {
        let pkg = packages.iter().find(|p| p.name == *pkg_name).unwrap();
        
        println!("{} Installing {}...", "→".cyan(), pkg_name.bold());
        
        let version_to_install = if let Some(ref v) = pkg.pinned_version {
            v.clone()
        } else {
            "latest".to_string()
        };

        // Run npm install
        match packages::npm_install(&pkg.name, &version_to_install) {
            Ok(()) => {
                println!("  {} {}@{} installed", "[OK]".green(), pkg.name, version_to_install);
                installed.push((pkg.name.clone(), version_to_install));
            }
            Err(e) => {
                println!("  {} Failed to install {}: {}", "[ERROR]".red(), pkg.name, e);
            }
        }
    }

    // Phase 7: Update ven.toml
    if !installed.is_empty() {
        println!();
        update_ven_toml_packages(&installed)?;
    }

    println!();
    Ok(())
}

/// Analyze a package and build dependency graph
fn analyze_package(
    pkg_name: &str,
    version_spec: &str,
    node_version: &str,
    skip_check: bool,
) -> Result<DependencyGraph> {
    use tokio::runtime::Runtime;

    let rt = Runtime::new()?;
    
    rt.block_on(async {
        let mut graph = DependencyGraph::new(node_version.to_string());
        graph.build(pkg_name, version_spec).await?;
        Ok(graph)
    })
}

/// Update ven.toml with multiple packages using proper TOML parsing
pub fn update_ven_toml_packages(packages: &[(String, String)]) -> Result<()> {
    use crate::core::find_ven_toml;

    let cwd = std::env::current_dir()?;
    let toml_path = find_ven_toml(&cwd)
        .ok_or_else(|| anyhow::anyhow!("No ven.toml found. Run: ven init"))?;

    // Read and parse TOML
    let content = std::fs::read_to_string(&toml_path)?;
    let mut doc: DocumentMut = content.parse::<DocumentMut>()
        .map_err(|e| anyhow::anyhow!("Failed to parse ven.toml: {}", e))?;

    // Ensure [packages] table exists
    if !doc.contains_key("packages") {
        doc["packages"] = toml_edit::table();
    }

    // Add or update each package
    let packages_table = doc["packages"].as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("Failed to access [packages] table"))?;

    for (pkg_name, version) in packages {
        let action = if packages_table.contains_key(pkg_name) {
            "Updated"
        } else {
            "Added"
        };

        packages_table.insert(pkg_name, value(version));
        
        println!("  {} {} {} = \"{}\"", "[TOML]".cyan(), action, pkg_name, version);
    }

    std::fs::write(&toml_path, doc.to_string())?;
    println!("  {} ven.toml updated with {} package(s)", "[OK]".green(), packages.len());

    Ok(())
}
