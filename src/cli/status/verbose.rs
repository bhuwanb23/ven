use crate::core::config::VenConfig;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;

use super::helpers::*;

pub(super) fn display_verbose_status(
    cwd: &Path,
    toml_path: &Path,
    config: &VenConfig,
    fix: bool,
) -> Result<()> {
    println!("\n  {} {}", "ven status".bold().cyan(), cwd.display());
    println!("  {} {}", "Config".dimmed(), toml_path.display());
    println!();

    // ── Runtime Status ──
    println!("  {}", "Runtime".bold().underline());

    if !config.runtime.node.is_empty() {
        let node_spec = &config.runtime.node;
        let resolved = resolve_version_for_display(node_spec)?;
        let installed = is_version_installed(node_spec);

        if installed {
            let bin_path = get_bin_path_for_version(node_spec)?;
            let version_size = calculate_dir_size(
                bin_path
                    .parent()
                    .ok_or_else(|| anyhow!("Bin path has no parent"))?,
            )?;

            println!(
                "    {} node {} ({})",
                "✓".green(),
                node_spec.bold(),
                resolved
            );
            println!("      {} {}", "Binary:".dimmed(), bin_path.display());
            println!("      {} {}", "Size:".dimmed(), format_bytes(version_size));

            // Check if active in PATH
            let is_active = check_if_version_active(node_spec)?;
            if is_active {
                println!("      {} {}", "Status:".dimmed(), "[ACTIVE]".green());
            } else {
                println!("      {} {}", "Status:".dimmed(), "[INACTIVE]".yellow());
            }
        } else {
            println!(
                "    {} node {} - {}",
                "✗".red(),
                node_spec.bold(),
                "not installed"
            );
            println!(
                "      {} Run: ven install node {}",
                "[!]".yellow(),
                node_spec
            );

            if fix {
                auto_install_version("node", node_spec)?;
            }
        }
    }
    if !config.runtime.python.is_empty() {
        let spec = &config.runtime.python;
        let installed = is_python_installed(spec);
        let resolved = resolve_python_for_display(spec)?;
        if installed {
            println!("    {} python {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!(
                "    {} python {} - {}",
                "✗".red(),
                spec.bold(),
                "not installed"
            );
            if fix {
                auto_install_version("python", spec)?;
            }
        }
    }
    if !config.runtime.go.is_empty() {
        let spec = &config.runtime.go;
        let installed = is_go_installed(spec);
        let resolved = resolve_go_for_display(spec)?;
        if installed {
            println!("    {} go {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!("    {} go {} - {}", "✗".red(), spec.bold(), "not installed");
            if fix {
                auto_install_version("go", spec)?;
            }
        }
    }
    if !config.runtime.rust.is_empty() {
        let spec = &config.runtime.rust;
        let installed = is_rust_installed(spec);
        let resolved = resolve_rust_for_display(spec)?;
        if installed {
            println!("    {} rust {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!(
                "    {} rust {} - {}",
                "✗".red(),
                spec.bold(),
                "not installed"
            );
            if fix {
                auto_install_version("rust", spec)?;
            }
        }
    }
    if !config.runtime.java.is_empty() {
        let spec = &config.runtime.java;
        let installed = is_java_installed(spec);
        let resolved = resolve_java_for_display(spec)?;
        if installed {
            println!("    {} java {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!(
                "    {} java {} - {}",
                "✗".red(),
                spec.bold(),
                "not installed"
            );
            if fix {
                auto_install_version("java", spec)?;
            }
        }
    }
    if !config.runtime.deno.is_empty() {
        let spec = &config.runtime.deno;
        let installed = is_deno_installed(spec);
        let resolved = resolve_deno_for_display(spec)?;
        if installed {
            println!("    {} deno {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!(
                "    {} deno {} - {}",
                "✗".red(),
                spec.bold(),
                "not installed"
            );
            if fix {
                auto_install_version("deno", spec)?;
            }
        }
    }
    if !config.runtime.ruby.is_empty() {
        let spec = &config.runtime.ruby;
        let installed = is_ruby_installed(spec);
        let resolved = resolve_ruby_for_display(spec)?;
        if installed {
            println!("    {} ruby {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!(
                "    {} ruby {} - {}",
                "✗".red(),
                spec.bold(),
                "not installed"
            );
            if fix {
                auto_install_version("ruby", spec)?;
            }
        }
    }
    if !config.runtime.bun.is_empty() {
        let spec = &config.runtime.bun;
        let installed = is_bun_installed(spec);
        let resolved = resolve_bun_for_display(spec)?;
        if installed {
            println!("    {} bun {} ({})", "✓".green(), spec.bold(), resolved);
        } else {
            println!(
                "    {} bun {} - {}",
                "✗".red(),
                spec.bold(),
                "not installed"
            );
            if fix {
                auto_install_version("bun", spec)?;
            }
        }
    }

    println!();

    // ── Package Status ──
    let pkg_count = config.packages.len();
    if pkg_count > 0 {
        println!("  {}", "Packages".bold().underline());

        if !config.runtime.deno.is_empty()
            && config.runtime.ruby.is_empty()
            && config.runtime.node.is_empty()
            && config.runtime.python.is_empty()
            && config.runtime.go.is_empty()
            && config.runtime.rust.is_empty()
            && config.runtime.java.is_empty()
            && config.runtime.bun.is_empty()
        {
            println!(
                "    {} Deno manages dependencies via imports/deno.json (ven does not install packages).",
                "[INFO]".cyan()
            );
            println!();
            return Ok(());
        }

        if !config.runtime.ruby.is_empty()
            && config.runtime.node.is_empty()
            && config.runtime.python.is_empty()
            && config.runtime.go.is_empty()
            && config.runtime.rust.is_empty()
            && config.runtime.java.is_empty()
            && config.runtime.deno.is_empty()
            && config.runtime.bun.is_empty()
        {
            println!(
                "    {} Ruby gems are managed via `ven add` and reconciled with Gemfile when present.",
                "[INFO]".cyan()
            );
        }

        let mut installed_count = 0;
        let mut missing_count = 0;
        let mut incompatible_count = 0;

        for (pkg_name, pkg_version) in &config.packages {
            let bun_runtime = !config.runtime.bun.is_empty()
                && config.runtime.node.is_empty()
                && config.runtime.python.is_empty()
                && config.runtime.go.is_empty()
                && config.runtime.rust.is_empty()
                && config.runtime.java.is_empty()
                && config.runtime.deno.is_empty()
                && config.runtime.ruby.is_empty();
            let is_installed = if bun_runtime {
                is_package_installed(pkg_name)
            } else {
                is_package_installed(pkg_name)
            };

            if is_installed {
                installed_count += 1;
                if let Ok(installed_ver) = get_installed_package_version(pkg_name) {
                    // Check compatibility
                    if let Ok(compatible) =
                        check_package_compatibility(pkg_name, &installed_ver, &config.runtime.node)
                    {
                        if compatible {
                            println!(
                                "    {} {}@{} {}",
                                "✓".green(),
                                pkg_name,
                                installed_ver,
                                "[compatible]".dimmed()
                            );
                        } else {
                            incompatible_count += 1;
                            println!(
                                "    {} {}@{} {}",
                                "⚠".yellow(),
                                pkg_name,
                                installed_ver,
                                "[incompatible]".yellow()
                            );
                        }
                    } else {
                        println!("    {} {}@{}", "✓".green(), pkg_name, installed_ver);
                    }

                    // Verbose: show more details (we're already in verbose mode)
                    let pkg_path = std::env::current_dir()
                        .unwrap_or_default()
                        .join("node_modules")
                        .join(pkg_name);
                    println!("      {} {}", "Location:".dimmed(), pkg_path.display());
                }
            } else {
                missing_count += 1;
                println!(
                    "    {} {}@{} {}",
                    "✗".red(),
                    pkg_name,
                    pkg_version,
                    "[not installed]".red()
                );

                if fix {
                    auto_install_package(pkg_name, pkg_version)?;
                }
            }
        }

        println!();
        println!(
            "    {} {} installed, {} missing, {} incompatible",
            "Summary:".dimmed(),
            installed_count.to_string().green(),
            missing_count.to_string().red(),
            incompatible_count.to_string().yellow()
        );

        if missing_count > 0 && !fix {
            println!("    {} Run: ven add <pkg>  or  npm install", "[TIP]".cyan());
        }
    }

    println!();

    // ── Environment Variables ──
    let env_count = config.env.len();
    if env_count > 0 {
        println!("  {}", "Environment".bold().underline());

        for (key, value) in &config.env {
            let key_str = key.as_str();
            let value_str = value.as_str();
            let current = std::env::var(key_str).ok();
            let is_set = current.as_deref() == Some(value_str);

            let icon = if is_set { "✓" } else { "○" };
            let status = if is_set {
                "[active]".green()
            } else {
                "[not set]".yellow()
            };

            println!(
                "    {} {}={} {}",
                icon,
                key_str.bold(),
                value_str.dimmed(),
                status
            );
        }
        println!();
    }

    // ── Health Summary ──
    println!("  {}", "Health Summary".bold().underline());
    print_health_summary(config)?;

    println!();
    Ok(())
}
