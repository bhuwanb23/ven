use crate::core::config::VenConfig;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use super::helpers::*;

pub(super) fn display_basic_status(
    cwd: &Path,
    toml_path: &Path,
    config: &VenConfig,
) -> Result<()> {
    println!("\n  {} {}", "ven status".bold(), cwd.display());
    println!("  {} {}", "Config".dimmed(), toml_path.display());
    println!();

    let has_node = !config.runtime.node.is_empty();
    let has_python = !config.runtime.python.is_empty();
    let has_go = !config.runtime.go.is_empty();
    let has_rust = !config.runtime.rust.is_empty();
    let has_java = !config.runtime.java.is_empty();
    let has_deno = !config.runtime.deno.is_empty();
    let has_bun = !config.runtime.bun.is_empty();
    let has_ruby = !config.runtime.ruby.is_empty();
    let has_any_runtime = has_node
        || has_python
        || has_go
        || has_rust
        || has_java
        || has_deno
        || has_bun
        || has_ruby;

    // Runtime section
    if has_node {
        let node_spec = &config.runtime.node;
        let resolved = resolve_version_for_display(node_spec)?;
        let installed = is_version_installed(node_spec);

        let status_icon = if installed { "✓" } else { "✗" };

        println!(
            "  {} node {} {}",
            status_icon,
            node_spec.bold(),
            format!("({})", resolved).dimmed()
        );

        if !installed {
            println!("    {} Run: ven install node {}", "[!]".yellow(), node_spec);
        }
    } else if !has_any_runtime {
        println!(
            "  {} {}",
            "[!]".yellow(),
            "no runtime pinned in [runtime]".dimmed()
        );
    }
    if has_python {
        let py_spec = &config.runtime.python;
        let resolved = resolve_python_for_display(py_spec)?;
        let installed = is_python_installed(py_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} python {} {}",
            status_icon,
            py_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!(
                "    {} Run: ven install python {}",
                "[!]".yellow(),
                py_spec
            );
        }
    }
    if has_go {
        let go_spec = &config.runtime.go;
        let resolved = resolve_go_for_display(go_spec)?;
        let installed = is_go_installed(go_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} go {} {}",
            status_icon,
            go_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!("    {} Run: ven install go {}", "[!]".yellow(), go_spec);
        }
    }
    if has_rust {
        let rust_spec = &config.runtime.rust;
        let resolved = resolve_rust_for_display(rust_spec)?;
        let installed = is_rust_installed(rust_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} rust {} {}",
            status_icon,
            rust_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!(
                "    {} Run: ven install rust {}",
                "[!]".yellow(),
                rust_spec
            );
        }
    }
    if has_java {
        let java_spec = &config.runtime.java;
        let resolved = resolve_java_for_display(java_spec)?;
        let installed = is_java_installed(java_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} java {} {}",
            status_icon,
            java_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!(
                "    {} Run: ven install java {}",
                "[!]".yellow(),
                java_spec
            );
        }
    }
    if has_deno {
        let deno_spec = &config.runtime.deno;
        let resolved = resolve_deno_for_display(deno_spec)?;
        let installed = is_deno_installed(deno_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} deno {} {}",
            status_icon,
            deno_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!(
                "    {} Run: ven install deno {}",
                "[!]".yellow(),
                deno_spec
            );
        }
    }
    if has_ruby {
        let ruby_spec = &config.runtime.ruby;
        let resolved = resolve_ruby_for_display(ruby_spec)?;
        let installed = is_ruby_installed(ruby_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} ruby {} {}",
            status_icon,
            ruby_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!(
                "    {} Run: ven install ruby {}",
                "[!]".yellow(),
                ruby_spec
            );
        }
    }
    if has_bun {
        let bun_spec = &config.runtime.bun;
        let resolved = resolve_bun_for_display(bun_spec)?;
        let installed = is_bun_installed(bun_spec);
        let status_icon = if installed { "✓" } else { "✗" };
        println!(
            "  {} bun {} {}",
            status_icon,
            bun_spec.bold(),
            format!("({})", resolved).dimmed()
        );
        if !installed {
            println!("    {} Run: ven install bun {}", "[!]".yellow(), bun_spec);
        }
    }

    // Packages section
    let pkg_count = config.packages.len();
    if pkg_count > 0 {
        // Count installed packages
        let ruby_only =
            has_ruby && !has_node && !has_python && !has_go && !has_rust && !has_java && !has_deno && !has_bun;
        let bun_only =
            has_bun && !has_node && !has_python && !has_go && !has_rust && !has_java && !has_deno && !has_ruby;
        let installed_count =
            if has_python && !has_node && !has_go && !has_rust && !has_java && !has_deno && !has_ruby
            {
                config
                    .packages
                    .keys()
                    .filter(|pkg| is_python_package_installed(pkg))
                    .count()
            } else if has_deno && !has_node && !has_python && !has_go && !has_rust && !has_java && !has_ruby
            {
                0
            } else if ruby_only {
                0
            } else if bun_only {
                config
                    .packages
                    .keys()
                    .filter(|pkg| is_package_installed(pkg))
                    .count()
            } else {
                config
                    .packages
                    .keys()
                    .filter(|pkg| is_package_installed(pkg))
                    .count()
            };

        println!(
            "  {} {} package(s) declared, {} installed",
            "packages".bold(),
            pkg_count,
            installed_count
        );

        // Show tip if packages are missing
        if installed_count < pkg_count {
            if has_deno && !has_node && !has_python && !has_go && !has_rust && !has_java && !has_ruby {
                println!(
                    "    {} Deno manages dependencies via imports/deno.json (ven does not install packages).",
                    "[TIP]".cyan()
                );
            } else if ruby_only {
                println!(
                    "    {} Ruby gems use Gemfile/Bundler — ven does not map [packages] to gem yet.",
                    "[TIP]".cyan()
                );
            } else if bun_only {
                println!("    {} Install missing: ven add <package>  (uses bun add)", "[TIP]".cyan());
            } else if has_python && !has_node && !has_go && !has_rust && !has_java && !has_deno && !has_ruby
            {
                println!("    {} Install missing: ven add <package>", "[TIP]".cyan());
            } else {
                println!(
                    "    {} Install missing: ven add --sync or npm install",
                    "[TIP]".cyan()
                );
            }
        }
    } else {
        println!("  {} {}", "packages".bold(), "none".dimmed());
    }

    // Environment variables
    let env_count = config.env.len();
    if env_count > 0 {
        println!("  {} {} variable(s) defined", "env".bold(), env_count);
    }

    println!();
    Ok(())
}
