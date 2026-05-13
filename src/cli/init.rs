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
    lang_flag: Option<&str>,
    ver_flag: Option<&str>,
    yes_flag: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let toml_path = cwd.join("ven.toml");

    if toml_path.exists() {
        return Err(anyhow::anyhow!("ven.toml already exists in this directory"));
    }

    // Headless / CI scaffold: --lang <node|python|...> [--ver X.Y.Z] [--yes]
    // Skips every interactive prompt and writes a minimal ven.toml.
    let auto = yes_flag || lang_flag.is_some() || !crate::core::runtime_bin::stdin_is_interactive();
    if auto {
        return cmd_init_headless(&cwd, &toml_path, lang_flag, ver_flag, validate);
    }

    print_installed_runtimes_banner()?;

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
                "Vue + Vite Frontend",
                "node",
                "20",
                vec![
                    ("vue", "^3.4.0"),
                    ("vite", "^5.0.0"),
                    ("@vitejs/plugin-vue", "^5.0.0"),
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
            let languages = vec![
                "node", "python", "go", "rust", "java", "deno", "bun", "ruby",
            ];
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
                "go" => select_go_version()?,
                "rust" => select_rust_version()?,
                "java" => select_java_version()?,
                "deno" => select_deno_version()?,
                "bun" => select_bun_version()?,
                "ruby" => select_ruby_version()?,
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
        let languages = vec![
            "node", "python", "go", "rust", "java", "deno", "bun", "ruby",
        ];
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
            "go" => select_go_version()?,
            "rust" => select_rust_version()?,
            "java" => select_java_version()?,
            "deno" => select_deno_version()?,
            "bun" => select_bun_version()?,
            "ruby" => select_ruby_version()?,
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

    if selected_language == "python" {
        content.push_str("\n[venv]\n");
        content.push_str(
            "# Optional. Hooks prepend `./venv` (or `./.venv`) when present; use `ven deactivate` (+ iex) to pause.\n",
        );
        content.push_str("auto_path = true\n");
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

    if selected_language == "python" {
        use crate::core::config::resolve_python_version;
        use crate::core::project_venv::{
            create_local_venv, ensure_gitignore_venv, PROJECT_VENV_DIR,
        };
        use crate::plugins::{LanguagePlugin, PythonPlugin};
        println!(
            "\n{} Creating local virtual environment ({}/)...",
            "[PY]".cyan().bold(),
            PROJECT_VENV_DIR
        );
        #[cfg(target_os = "windows")]
        {
            let plugin = PythonPlugin;
            let installed = plugin.list_installed().unwrap_or_default();
            match resolve_python_version(&selected_version, &installed) {
                Ok(resolved) => match plugin.bin_path(&resolved) {
                    Ok(bin) => {
                        let py_exe = bin.join("python.exe");
                        if !py_exe.is_file() {
                            println!(
                                "  {} No python.exe at {}. Run: {}",
                                "[!]".yellow().bold(),
                                py_exe.display(),
                                format!("ven install python {}", resolved).bold()
                            );
                        } else {
                            match create_local_venv(&cwd, &py_exe) {
                                Ok(venv_path) => {
                                    ensure_gitignore_venv(&cwd)?;
                                    println!(
                                        "  {} `{}/` at {}",
                                        "[OK]".green().bold(),
                                        PROJECT_VENV_DIR,
                                        venv_path.display()
                                    );
                                    print_python_venv_usage_hints();
                                }
                                Err(e) => {
                                    println!(
                                        "  {} Could not create `{}`: {}",
                                        "[!]".yellow().bold(),
                                        PROJECT_VENV_DIR,
                                        e
                                    );
                                    println!(
                                        "    Try: {} -m venv {}  (or ven installs virtualenv if stdlib venv is missing)",
                                        py_exe.display(),
                                        PROJECT_VENV_DIR
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => println!(
                        "  {} Python {} path error: {}",
                        "[!]".yellow().bold(),
                        resolved,
                        e
                    ),
                },
                Err(_) => {
                    println!(
                        "  {} No installed Python matches `{}`.",
                        "[!]".yellow().bold(),
                        selected_version
                    );
                    println!(
                        "    Run: {}  then open a new shell (or {})",
                        format!("ven install python {}", selected_version).bold(),
                        "ven-use".bold()
                    );
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            use std::path::Path;
            use std::process::Command;
            let ok = Command::new("python3")
                .arg("--version")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                match create_local_venv(&cwd, Path::new("python3")) {
                    Ok(venv_path) => {
                        ensure_gitignore_venv(&cwd)?;
                        println!(
                            "  {} `{}/` at {}",
                            "[OK]".green().bold(),
                            PROJECT_VENV_DIR,
                            venv_path.display()
                        );
                        print_python_venv_usage_hints();
                    }
                    Err(e) => println!("  {} {}", "[!]".yellow().bold(), e),
                }
            } else {
                println!(
                    "  {} python3 not on PATH. Create a venv with: python3 -m venv {}",
                    "[!]".yellow().bold(),
                    PROJECT_VENV_DIR
                );
            }
        }
    }

    // MODE 4: Validation
    if validate {
        println!("\n{} Running validation...", "🔍".bold().cyan());
        run_validation(
            &selected_language,
            &selected_version,
            &selected_packages,
            &cwd,
        )?;
    } else {
        println!("\nEdit this file to customize your dependencies.");
        if selected_language != "python" {
            println!(
                "Run: ven install {} {}   if you still need that runtime under ven",
                selected_language, selected_version
            );
        }
        println!(
            "Apply this folder in your shell: {}  (after one-time  ven setup  or  ven shell install)",
            "ven-use".bold()
        );
    }

    Ok(())
}

/// Non-interactive `ven init`. Picks the newest installed version of the
/// requested language (or asks the user via `--lang`/`--ver` flags). Used by
/// CI, the SDK, and any context where a TTY isn't available.
fn cmd_init_headless(
    cwd: &std::path::Path,
    toml_path: &std::path::Path,
    lang_flag: Option<&str>,
    ver_flag: Option<&str>,
    validate: bool,
) -> Result<()> {
    use crate::plugins::PluginRegistry;

    let lang = lang_flag.map(|s| s.to_ascii_lowercase()).unwrap_or_else(|| {
        // Fallback: pick the first language with an installed runtime.
        let registry = PluginRegistry::new();
        for candidate in registry.list_languages() {
            if let Ok(plug) = registry.require(candidate) {
                if plug.list_installed().map(|v| !v.is_empty()).unwrap_or(false) {
                    return candidate.to_string();
                }
            }
        }
        "node".to_string()
    });

    let registry = PluginRegistry::new();
    let plugin = registry.require(&lang).map_err(|_| {
        anyhow::anyhow!(
            "Unknown language `{lang}`. Supported: node, python, go, rust, java, deno, bun, ruby"
        )
    })?;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No `{lang}` runtime installed under ven. Run: `ven install {lang} latest` first."
        );
    }

    let version = match ver_flag {
        Some(v) if !v.trim().is_empty() => {
            // Trust the caller; use as-is even if not currently installed (`ven sync`
            // can install missing toolchains later).
            v.trim().to_string()
        }
        _ => installed.last().cloned().unwrap_or_else(|| "latest".to_string()),
    };

    let mut content = String::from("[runtime]\n");
    content.push_str(&format!("{} = \"{}\"\n", lang, version));
    content.push_str("\n[packages]\n");
    content.push_str("# Add your dependencies here, e.g.  ven add express\n");
    if lang == "python" {
        content.push_str("\n[venv]\nauto_path = true\n");
    }
    std::fs::write(toml_path, &content)?;

    println!(
        "{} Created {} ({} {})",
        "✓".green(),
        toml_path.display(),
        lang.bold(),
        version.green()
    );

    if lang == "python" {
        if let Err(e) = bootstrap_local_python_venv(cwd, &version) {
            println!("  {} venv bootstrap skipped: {}", "[!]".yellow(), e);
        }
    }

    if validate {
        run_validation(&lang, &version, &[], cwd)?;
    }
    Ok(())
}

/// Auto-create `./venv` for Python projects in headless mode. Mirrors what the
/// interactive path does, just without prompts or hint banners.
fn bootstrap_local_python_venv(cwd: &std::path::Path, requested_version: &str) -> Result<()> {
    use crate::core::config::resolve_python_version;
    use crate::core::project_venv::{create_local_venv, ensure_gitignore_venv, PROJECT_VENV_DIR};
    use crate::plugins::{LanguagePlugin, PythonPlugin};

    let plugin = PythonPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    let resolved = resolve_python_version(requested_version, &installed)?;
    let bin = plugin.bin_path(&resolved)?;
    #[cfg(target_os = "windows")]
    let py_exe = bin.join("python.exe");
    #[cfg(not(target_os = "windows"))]
    let py_exe = bin.join("python");
    if !py_exe.is_file() {
        anyhow::bail!("python interpreter not found at {}", py_exe.display());
    }
    let venv_path = create_local_venv(cwd, &py_exe)?;
    let _ = ensure_gitignore_venv(cwd);
    println!(
        "  {} `{}/` created at {}",
        "[OK]".green(),
        PROJECT_VENV_DIR,
        venv_path.display()
    );
    Ok(())
}

#[cfg(target_os = "windows")]
fn print_python_venv_usage_hints() {
    use crate::core::project_venv::PROJECT_VENV_DIR;
    println!(
        "  {} Project env folder is `{}/` (same as most tutorials; legacy `./.venv` is still detected).",
        "[PY]".cyan().bold(),
        PROJECT_VENV_DIR
    );
    println!(
        "      PowerShell:  {}",
        format!("& .\\{}\\Scripts\\Activate.ps1", PROJECT_VENV_DIR).dimmed()
    );
    println!(
        "      cmd.exe:     {}",
        format!("{}\\Scripts\\activate.bat", PROJECT_VENV_DIR).dimmed()
    );
    println!(
        "      Or rely on `{}` here (prepends `{}/Scripts` to PATH).",
        "ven-use".bold(),
        PROJECT_VENV_DIR
    );
    println!(
        "      {}",
        "If you run `ven deactivate`, run `ven-use` again to put `./venv/Scripts` + ven Python back."
            .dimmed()
    );
}

#[cfg(not(target_os = "windows"))]
fn print_python_venv_usage_hints() {
    use crate::core::project_venv::PROJECT_VENV_DIR;
    println!(
        "  {} Activate:  {}",
        "[PY]".cyan().bold(),
        format!("source {}/bin/activate", PROJECT_VENV_DIR).dimmed()
    );
    println!(
        "      Or: `{}` in this repo. After `ven deactivate`, run `ven-use` again to prepend `{}/`.",
        "ven-use".bold(),
        PROJECT_VENV_DIR
    );
}

/// Show Node and Python versions already installed under ven before choosing a project runtime.
fn print_installed_runtimes_banner() -> Result<()> {
    use crate::plugins::PluginRegistry;

    let registry = PluginRegistry::new();
    println!(
        "\n{} Installed runtimes (ven-managed):",
        "[INSTALLED]".cyan().bold()
    );
    for lang in registry.list_languages() {
        let plugin = registry.require(lang)?;
        let versions = plugin.list_installed().unwrap_or_default();
        if versions.is_empty() {
            println!("  {} {} — {}", "•".dimmed(), lang.bold(), "(none)".dimmed());
        } else {
            println!(
                "  {} {} — {}",
                "•".dimmed(),
                lang.bold(),
                versions.join(", ").cyan()
            );
        }
    }
    println!();
    Ok(())
}

/// Interactive node version selection — **only** versions already installed under ven.
fn select_node_version() -> Result<String> {
    use crate::plugins::{LanguagePlugin, NodePlugin};

    let theme = ColorfulTheme::default();
    let plugin = NodePlugin;
    let installed = plugin.list_installed().unwrap_or_default();

    if installed.is_empty() {
        anyhow::bail!(
            "No node versions installed under ven.\n\
             Install one first, e.g.:  ven install node latest\n\
             Then run  ven init  again."
        );
    }

    struct VersionOption {
        value: String,
        display: String,
    }

    let options: Vec<VersionOption> = installed
        .iter()
        .map(|version| VersionOption {
            value: version.clone(),
            display: format!("{}  {}", version, get_version_info(version)),
        })
        .collect();

    let display_items: Vec<String> = options.iter().map(|o| o.display.clone()).collect();

    let version_idx = Select::with_theme(&theme)
        .with_prompt("Select node version (installed)")
        .items(&display_items)
        .default(0)
        .interact()?;

    Ok(options[version_idx].value.clone())
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

/// Interactive Python version selection — **only** versions already installed under ven.
fn select_python_version() -> Result<String> {
    use crate::plugins::{LanguagePlugin, PythonPlugin};

    let theme = ColorfulTheme::default();
    let plugin = PythonPlugin;
    let installed = plugin.list_installed().unwrap_or_default();

    if installed.is_empty() {
        anyhow::bail!(
            "No Python versions installed under ven.\n\
             Install one first (Windows embeddable), e.g.:  ven install python 3.12.7\n\
             Then run  ven init  again."
        );
    }

    struct VersionOption {
        value: String,
        display: String,
    }

    let options: Vec<VersionOption> = installed
        .iter()
        .map(|version| VersionOption {
            value: version.clone(),
            display: format!("{}  {}", version, get_python_version_info(version)),
        })
        .collect();

    let display_items: Vec<String> = options.iter().map(|o| o.display.clone()).collect();

    let version_idx = Select::with_theme(&theme)
        .with_prompt("Select Python version (installed)")
        .items(&display_items)
        .default(0)
        .interact()?;

    Ok(options[version_idx].value.clone())
}

/// Short hint beside each Python patch version (same lines as `ven list` semantics).
fn get_python_version_info(version: &str) -> String {
    let minor = version
        .split('.')
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    match minor {
        0..=7 => "⚠️  EOL line".to_string(),
        8 | 9 => "🔒 Security fixes only".to_string(),
        10 => "🔧 Maintenance / security".to_string(),
        11 | 12 => "✅ Stable bugfix line".to_string(),
        13..=99 => "🔥 Newer 3.x".to_string(),
        _ => "✅ Installed".to_string(),
    }
}

/// Interactive Go version selection — only versions installed under ven.
fn select_go_version() -> Result<String> {
    use crate::plugins::{GoPlugin, LanguagePlugin};
    let theme = ColorfulTheme::default();
    let plugin = GoPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No Go versions installed under ven.\n\
             Install one first, e.g.:  ven install go 1.21.5\n\
             Then run  ven init  again."
        );
    }
    let items: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  {}", v, "Go toolchain".dimmed()))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select Go version (installed)")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}

/// Interactive Rust version selection — only versions installed under ven.
fn select_rust_version() -> Result<String> {
    use crate::plugins::{LanguagePlugin, RustPlugin};
    let theme = ColorfulTheme::default();
    let plugin = RustPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No Rust versions installed under ven.\n\
             Install one first, e.g.:  ven install rust 1.75.0\n\
             Then run  ven init  again."
        );
    }
    let items: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  {}", v, "Rust toolchain".dimmed()))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select Rust version (installed)")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}

/// Interactive Java version selection — only versions installed under ven.
fn select_java_version() -> Result<String> {
    use crate::plugins::{JavaPlugin, LanguagePlugin};
    let theme = ColorfulTheme::default();
    let plugin = JavaPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No Java versions installed under ven.\n\
             Install one first, e.g.:  ven install java 17\n\
             Then run  ven init  again."
        );
    }
    let items: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  {}", v, "OpenJDK".dimmed()))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select Java version (installed)")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}

/// Interactive Deno version selection — only versions installed under ven.
fn select_deno_version() -> Result<String> {
    use crate::plugins::{DenoPlugin, LanguagePlugin};
    let theme = ColorfulTheme::default();
    let plugin = DenoPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No Deno versions installed under ven.\n\
             Install one first, e.g.:  ven install deno latest\n\
             Then run  ven init  again."
        );
    }
    let items: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  {}", v, "Deno runtime".dimmed()))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select Deno version (installed)")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}

/// Interactive Bun version selection — only versions installed under ven.
fn select_bun_version() -> Result<String> {
    use crate::plugins::{BunPlugin, LanguagePlugin};
    let theme = ColorfulTheme::default();
    let plugin = BunPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No Bun versions installed under ven.\n\
             Install one first, e.g.:  ven install bun latest\n\
             Then run  ven init  again."
        );
    }
    let items: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  {}", v, "Bun runtime".dimmed()))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select Bun version (installed)")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}

/// Interactive Ruby version selection — only versions installed under ven.
fn select_ruby_version() -> Result<String> {
    use crate::plugins::{LanguagePlugin, RubyPlugin};
    let theme = ColorfulTheme::default();
    let plugin = RubyPlugin;
    let installed = plugin.list_installed().unwrap_or_default();
    if installed.is_empty() {
        anyhow::bail!(
            "No Ruby versions installed under ven.\n\
             Install one first, e.g.:  ven install ruby latest\n\
             Then run  ven init  again."
        );
    }
    let items: Vec<String> = installed
        .iter()
        .map(|v| format!("{}  {}", v, "MRI Ruby".dimmed()))
        .collect();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select Ruby version (installed)")
        .items(&items)
        .default(0)
        .interact()?;
    Ok(installed[idx].clone())
}

/// Health check & validation system
fn run_validation(
    language: &str,
    version: &str,
    packages: &[(String, String)],
    project_dir: &std::path::Path,
) -> Result<()> {
    use crate::plugins::{LanguagePlugin, NodePlugin};

    let mut all_checks_passed = true;

    // Check 1: ven.toml created
    println!("  {} ven.toml created", "✓".green());

    // Check 2: runtime version installed
    if language == "node" {
        let plugin = NodePlugin;
        let installed = plugin.list_installed().unwrap_or_default();

        if version == "latest" || version == "lts" || !version.contains('.') {
            // Alias - will be resolved during install
            println!(
                "  {} node {} (will resolve during install)",
                "⚠️".yellow(),
                version
            );
        } else if installed.contains(&version.to_string()) {
            println!("  {} node {} installed", "✓".green(), version);
        } else {
            println!("  {} node {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install node {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    if language == "python" {
        use crate::core::project_venv::{local_venv_bin_dir, PROJECT_VENV_DIR};
        if local_venv_bin_dir(project_dir).is_some() {
            println!(
                "  {} `{}/` (or legacy `.venv`) is present",
                "✓".green(),
                PROJECT_VENV_DIR
            );
        } else {
            println!("  {} `{}/` not found", "✗".red(), PROJECT_VENV_DIR);
            println!(
                "    {} Run: ven install python {}  (Windows) or python3 -m venv {}",
                "💡".yellow(),
                version,
                PROJECT_VENV_DIR
            );
            all_checks_passed = false;
        }
    }

    if language == "go" {
        use crate::plugins::{GoPlugin, LanguagePlugin};
        let plugin = GoPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        if installed.contains(&version.to_string()) {
            println!("  {} Go {} installed", "✓".green(), version);
        } else {
            println!("  {} Go {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install go {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    if language == "rust" {
        use crate::plugins::{LanguagePlugin, RustPlugin};
        let plugin = RustPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        if installed.contains(&version.to_string()) {
            println!("  {} Rust {} installed", "✓".green(), version);
        } else {
            println!("  {} Rust {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install rust {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    if language == "java" {
        use crate::plugins::{JavaPlugin, LanguagePlugin};
        let plugin = JavaPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        if installed.contains(&version.to_string()) {
            println!("  {} Java {} installed", "✓".green(), version);
        } else {
            println!("  {} Java {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install java {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    if language == "deno" {
        use crate::plugins::{DenoPlugin, LanguagePlugin};
        let plugin = DenoPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        if installed.contains(&version.to_string()) {
            println!("  {} Deno {} installed", "✓".green(), version);
        } else {
            println!("  {} Deno {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install deno {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    if language == "bun" {
        use crate::plugins::{BunPlugin, LanguagePlugin};
        let plugin = BunPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        if installed.contains(&version.to_string()) {
            println!("  {} Bun {} installed", "✓".green(), version);
        } else {
            println!("  {} Bun {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install bun {}", "💡".yellow(), version);
            all_checks_passed = false;
        }
    }

    if language == "ruby" {
        use crate::plugins::{LanguagePlugin, RubyPlugin};
        let plugin = RubyPlugin;
        let installed = plugin.list_installed().unwrap_or_default();
        if installed.contains(&version.to_string()) {
            println!("  {} Ruby {} installed", "✓".green(), version);
        } else {
            println!("  {} Ruby {} not installed yet", "✗".red(), version);
            println!("    {} Run: ven install ruby {}", "💡".yellow(), version);
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
