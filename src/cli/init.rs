use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use std::fs;

/// Interactive project initialization with templates, packages, and validation
pub fn cmd_init(
    _node: Option<&str>,
    use_template: bool,
    with_packages: bool,
    validate: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let toml_path = cwd.join("ven.toml");

    if toml_path.exists() {
        return Err(anyhow::anyhow!("ven.toml already exists in this directory"));
    }

    let theme = ColorfulTheme::default();
    let selected_language: String;
    let selected_version: String;
    let mut selected_packages: Vec<(String, String)> = Vec::new();
    let mut template_name = String::new();

    // MODE 1: Template selection
    if use_template {
        println!("\n{} Smart Project Templates", "📦".bold().cyan());

        let templates = vec![
            (
                "Express API Server",
                "node",
                "20",
                vec![
                    ("express", "^4.18.2"),
                    ("cors", "^2.8.5"),
                    ("dotenv", "^16.3.1"),
                ],
            ),
            (
                "React + Vite Frontend",
                "node",
                "20",
                vec![
                    ("react", "^18.2.0"),
                    ("react-dom", "^18.2.0"),
                    ("vite", "^5.0.0"),
                ],
            ),
            (
                "Next.js Full-stack",
                "node",
                "20",
                vec![
                    ("next", "^14.0.0"),
                    ("react", "^18.2.0"),
                    ("react-dom", "^18.2.0"),
                ],
            ),
            ("Empty Project", "", "", vec![]),
        ];

        let template_names: Vec<&str> = templates.iter().map(|t| t.0).collect();

        let template_idx = Select::with_theme(&theme)
            .with_prompt("Select project template")
            .items(&template_names)
            .default(0)
            .interact()?;

        let template = &templates[template_idx];
        template_name = template.0.to_string();

        // Check if this is an "Empty Project" - if so, do interactive language/version selection
        if template.1.is_empty() {
            println!("\n{} Selected: {}", "✓".green(), template_name.bold());
            println!("{} Configuring your empty project...", "→".cyan());

            // Show language selection
            let languages = vec!["node", "python"];
            let language_idx = Select::with_theme(&theme)
                .with_prompt("Select language")
                .items(&languages)
                .default(0)
                .interact()?;

            selected_language = languages[language_idx].to_string();

            // Version selection
            selected_version = match selected_language.as_str() {
                "node" => select_node_version()?,
                "python" => select_python_version()?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported language: {}",
                        selected_language
                    ));
                }
            };
        } else {
            // Pre-configured template
            selected_language = template.1.to_string();
            selected_version = template.2.to_string();

            // Add template packages
            for (pkg, ver) in &template.3 {
                selected_packages.push((pkg.to_string(), ver.to_string()));
            }

            println!("\n{} Selected: {}", "✓".green(), template_name.bold());
        }
    } else {
        // MODE 2: Interactive language & version selection
        let languages = vec!["node", "python"];
        let language_idx = Select::with_theme(&theme)
            .with_prompt("Select language")
            .items(&languages)
            .default(0)
            .interact()?;

        selected_language = languages[language_idx].to_string();

        // Version selection
        selected_version = match selected_language.as_str() {
            "node" => select_node_version()?,
            "python" => select_python_version()?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported language: {}",
                    selected_language
                ));
            }
        };
    }

    // MODE 3: Interactive package selection
    if with_packages && selected_packages.is_empty() {
        println!("\n{} Interactive Package Selection", "📦".bold().cyan());

        let popular_packages = vec![
            ("express", "^4.18.2", "Fast, minimalist web framework"),
            ("typescript", "^5.3.0", "Typed JavaScript for better DX"),
            ("dotenv", "^16.3.1", "Load environment variables from .env"),
            ("cors", "^2.8.5", "Enable CORS support"),
            ("morgan", "^1.10.0", "HTTP request logger"),
            ("jest", "^29.7.0", "Testing framework"),
            ("eslint", "^8.56.0", "Code linting and quality"),
            ("prettier", "^3.1.0", "Code formatting"),
        ];

        let pkg_display: Vec<String> = popular_packages
            .iter()
            .map(|(name, ver, desc)| format!("{} {} - {}", name.bold(), ver.dimmed(), desc))
            .collect();

        let selected_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Add popular packages? (SPACE to select, ENTER to continue)")
            .items(&pkg_display)
            .interact()?;

        for idx in selected_indices {
            let pkg = &popular_packages[idx];
            selected_packages.push((pkg.0.to_string(), pkg.1.to_string()));
        }

        if !selected_packages.is_empty() {
            println!(
                "\n{} Selected {} packages",
                "✓".green(),
                selected_packages.len()
            );
        }
    }

    // Generate ven.toml
    let mut content = String::from("[runtime]\n");
    content.push_str(&format!(
        "{} = \"{}\"\n",
        selected_language, selected_version
    ));

    content.push_str("\n[packages]\n");
    if selected_packages.is_empty() {
        content.push_str("# Add your dependencies here\n");
        content.push_str("# express = \"^4.18.2\"\n");
    } else {
        for (pkg, ver) in &selected_packages {
            content.push_str(&format!("{} = \"{}\"\n", pkg, ver));
        }
    }

    fs::write(&toml_path, &content)?;

    // Success message
    println!("\n{} Created {}", "✓".green(), toml_path.display());
    if !template_name.is_empty() {
        println!("  {} Template: {}", "📦".cyan(), template_name.bold());
    }
    println!(
        "  {} {} {}",
        "🔧".cyan(),
        selected_language.bold(),
        selected_version.green()
    );

    if !selected_packages.is_empty() {
        println!("  {} {} packages:", "📦".cyan(), selected_packages.len());
        for (pkg, ver) in &selected_packages {
            println!("      {} {}", pkg.bold(), ver.dimmed());
        }
    }

    // MODE 4: Validation
    if validate {
        println!("\n{} Running validation...", "🔍".bold().cyan());
        run_validation(&selected_language, &selected_version, &selected_packages)?;
    } else {
        println!("\nEdit this file to customize your dependencies.");
        println!(
            "Run: ven install {} {}   to install this version",
            selected_language, selected_version
        );
    }

    Ok(())
}

/// Interactive Node.js version selection with compatibility matrix
fn select_node_version() -> Result<String> {
    use crate::plugins::{LanguagePlugin, NodePlugin};

    let theme = ColorfulTheme::default();
    let plugin = NodePlugin;

    // Get installed versions
    let installed = plugin.list_installed().unwrap_or_default();

    // Build version options with metadata
    struct VersionOption {
        value: String,
        display: String,
    }

    let mut options: Vec<VersionOption> = Vec::new();

    // Add installed versions with compatibility info
    for version in &installed {
        let info = get_version_info(version);
        options.push(VersionOption {
            value: version.clone(),
            display: format!("{}  {}", version, info),
        });
    }

    // Add separator if there are installed versions
    if !installed.is_empty() {
        options.push(VersionOption {
            value: "".to_string(),
            display: "─── Version Aliases ───".to_string(),
        });
    }

    // Add aliases with descriptions
    options.push(VersionOption {
        value: "latest".to_string(),
        display: "latest              Latest stable release".to_string(),
    });
    options.push(VersionOption {
        value: "lts".to_string(),
        display: "lts                 Latest LTS (recommended)".to_string(),
    });
    options.push(VersionOption {
        value: "22".to_string(),
        display: "22                  Current release line".to_string(),
    });
    options.push(VersionOption {
        value: "20".to_string(),
        display: "20                  Active LTS (best compatibility)".to_string(),
    });
    options.push(VersionOption {
        value: "18".to_string(),
        display: "18                  Maintenance LTS".to_string(),
    });

    // If no installed versions, show informative message
    if installed.is_empty() {
        options.insert(
            0,
            VersionOption {
                value: "".to_string(),
                display: "⚠️  No versions installed - select an alias to install".to_string(),
            },
        );
    }

    // Extract display items
    let display_items: Vec<String> = options.iter().map(|opt| opt.display.clone()).collect();

    let version_idx = Select::with_theme(&theme)
        .with_prompt("Select Node.js version")
        .items(&display_items)
        .default(if installed.is_empty() { 2 } else { 0 })
        .interact()?;

    let selected = &options[version_idx];

    // Skip separator and warning
    if selected.value.is_empty() || selected.value.starts_with("⚠️") {
        return Err(anyhow::anyhow!("Please select a valid version"));
    }

    Ok(selected.value.clone())
}

/// Get version compatibility and status information
fn get_version_info(version: &str) -> String {
    let major = version.split('.').next().unwrap_or("0");
    let major_num: u32 = major.parse().unwrap_or(0);

    // Determine version status and compatibility
    if major_num >= 23 {
        format!("🔥 Current  (~85% pkg compat)")
    } else if major_num == 22 {
        format!("✅ Current  (~95% pkg compat)")
    } else if major_num == 20 {
        format!("⭐ LTS     (~98% pkg compat) [Recommended]")
    } else if major_num == 18 {
        format!("🔧 LTS     (~95% pkg compat) [Maintenance]")
    } else if major_num <= 16 {
        format!("⚠️  Deprecated (<80% pkg compat)")
    } else {
        format!("✅ Installed")
    }
}

/// Interactive Python version selection (stub - future implementation)
fn select_python_version() -> Result<String> {
    let theme = ColorfulTheme::default();
    let versions = vec!["3.12.2", "3.11.8", "3.10.14", "latest"];

    let version_idx = Select::with_theme(&theme)
        .with_prompt("Select Python version")
        .items(&versions)
        .default(0)
        .interact()?;

    Ok(versions[version_idx].to_string())
}

/// Health check & validation system
fn run_validation(language: &str, version: &str, packages: &[(String, String)]) -> Result<()> {
    use crate::plugins::{LanguagePlugin, NodePlugin};

    let mut all_checks_passed = true;

    // Check 1: ven.toml created
    println!("  {} ven.toml created", "✓".green());

    // Check 2: Node.js version installed
    if language == "node" {
        let plugin = NodePlugin;
        let installed = plugin.list_installed().unwrap_or_default();

        if version == "latest" || version == "lts" || !version.contains('.') {
            // Alias - will be resolved during install
            println!(
                "  {} Node.js {} (will resolve during install)",
                "⚠️".yellow(),
                version
            );
        } else if installed.contains(&version.to_string()) {
            println!("  {} Node.js {} installed", "✓".green(), version);
        } else {
            println!("  {} Node.js {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install node {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    // Check 3: Package compatibility (basic check)
    if !packages.is_empty() {
        println!("  {} {} packages declared", "✓".green(), packages.len());

        // Future: Check npm registry for compatibility
        // For now, just list them
        for (pkg, _) in packages {
            println!("      {} {}", "•".dimmed(), pkg);
        }
    }

    // Check 4: Environment variables
    println!("  {} Environment variables (optional)", "ℹ️".blue());

    // Final summary
    println!();
    if all_checks_passed {
        println!("  {} Ready to develop!", "🚀".green().bold());
    } else {
        println!("  {} Some issues need attention", "⚠️".yellow().bold());
    }
    println!();

    Ok(())
}
