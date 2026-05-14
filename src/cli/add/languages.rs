use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use super::update_ven_toml_packages;

use crate::core::deno_imports::{self, DenoManifest};
use crate::core::gemfile::Gemfile;
use crate::core::java_manifest::{self, JavaCoord};
use crate::core::project_venv::local_venv_bin_dir;
use crate::core::requirements::{requirement_from_spec, Requirements};
use crate::core::ruby_gems;
use crate::core::runtime_bin::runtime_tool;

pub(super) fn cmd_add_python(package_specs: &[String], dry_run: bool) -> Result<()> {
    let mut installed = Vec::new();
    println!("\n{}", "ven add (python)".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            let (name, declared) = parse_python_spec(spec);
            println!("  {} {} => {}", "[PREVIEW]".cyan(), name.bold(), declared);
        }
        println!();
        return Ok(());
    }

    ensure_project_python_venv()?;

    for spec in package_specs {
        let (name, declared) = parse_python_spec(spec);
        let python = resolve_python_cmd();
        let status = Command::new(&python)
            .args(["-m", "pip", "install", spec])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!(
                    "  {} {}",
                    "[OK]".green(),
                    format!("Installed {}", spec.bold())
                );
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to install {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
        sync_requirements_after_add(package_specs)?;
    }
    println!();
    Ok(())
}

/// Mirror successful `pip install <spec>` calls into `requirements.txt`.
fn sync_requirements_after_add(specs: &[String]) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mut req = Requirements::load_or_empty(&cwd)?;
    for spec in specs {
        let (name, raw) = requirement_from_spec(spec);
        req.upsert(&name, &raw);
    }
    req.write()?;
    println!(
        "  {} {}",
        "[REQ]".cyan(),
        format!("Synced requirements.txt").dimmed()
    );
    Ok(())
}

pub(super) fn cmd_add_go(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (go)".bold().cyan());
    println!("  {} {} module(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            println!("  {} go get {}", "[PREVIEW]".cyan(), spec.bold());
        }
        println!();
        return Ok(());
    }

    ensure_go_mod()?;
    let go_bin = runtime_tool("go", "go");
    let mut installed = Vec::new();
    for spec in package_specs {
        let status = Command::new(&go_bin).args(["get", spec]).status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} {}", "[OK]".green(), format!("Added {}", spec.bold()));
                let (name, declared) = parse_go_spec(spec);
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to add {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_rust(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (rust)".bold().cyan());
    println!("  {} {} crate(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            println!("  {} cargo add {}", "[PREVIEW]".cyan(), spec.bold());
        }
        println!();
        return Ok(());
    }

    ensure_cargo_manifest()?;
    let cargo_bin = runtime_tool("rust", "cargo");
    let mut installed = Vec::new();
    for spec in package_specs {
        let status = Command::new(&cargo_bin).args(["add", spec]).status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} {}", "[OK]".green(), format!("Added {}", spec.bold()));
                let (name, declared) = parse_rust_spec(spec);
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to add {}", "[ERROR]".red(), spec),
            Err(e) => println!("  {} {}", "[ERROR]".red(), e),
        }
    }
    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_java(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (java)".bold().cyan());
    println!("  {} {} item(s)", "[PLAN]".cyan(), package_specs.len());

    let cwd = std::env::current_dir()?;
    let project = match java_manifest::detect(&cwd) {
        Some(p) => p,
        None => {
            println!(
                "  {} No pom.xml or build.gradle(.kts) found.",
                "[ERROR]".red()
            );
            println!(
                "  {} Run `ven init --template` or create a Maven/Gradle project first.",
                "[TIP]".cyan()
            );
            println!();
            return Ok(());
        }
    };

    if dry_run {
        println!(
            "  {} {} detected: {}",
            "[INFO]".cyan(),
            format!("{:?}", project.tool).bold(),
            project.manifest.display()
        );
        for spec in package_specs {
            match JavaCoord::parse(spec) {
                Ok(c) => println!(
                    "  {} {}:{} version {:?}",
                    "[PREVIEW]".cyan(),
                    c.group.bold(),
                    c.artifact.bold(),
                    c.version
                ),
                Err(e) => println!("  {} {}: {}", "[ERROR]".red(), spec, e),
            }
        }
        println!();
        return Ok(());
    }

    let mut installed = Vec::new();
    for spec in package_specs {
        let coord = match JavaCoord::parse(spec) {
            Ok(c) => c,
            Err(e) => {
                println!("  {} {}: {}", "[ERROR]".red(), spec, e);
                continue;
            }
        };
        match java_manifest::add(&project, &coord) {
            Ok(()) => {
                println!(
                    "  {} {}",
                    "[OK]".green(),
                    format!(
                        "Added {}:{}{}",
                        coord.group,
                        coord.artifact,
                        coord
                            .version
                            .as_ref()
                            .map(|v| format!(":{v}"))
                            .unwrap_or_default()
                    )
                    .bold()
                );
                installed.push((
                    coord.ven_toml_key(),
                    coord
                        .version
                        .clone()
                        .unwrap_or_else(|| "latest".to_string()),
                ));
            }
            Err(e) => println!("  {} {}: {}", "[ERROR]".red(), spec, e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_ruby(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (ruby)".bold().cyan());
    println!("  {} {} gem(s)", "[PLAN]".cyan(), package_specs.len());

    let cwd = std::env::current_dir()?;
    let gemfile_path = Gemfile::path_for(&cwd);
    let use_bundler = gemfile_path.is_file() && which_bundle();

    if dry_run {
        println!(
            "  {} Dry run mode — no changes will be made",
            "[DRY-RUN]".yellow()
        );
        if use_bundler {
            println!(
                "  {} Gemfile detected — would use `bundle add`",
                "[INFO]".cyan()
            );
        }
        println!();
        for spec in package_specs {
            let (name, declared) = parse_ruby_gem_spec(spec);
            let decl = declared.as_deref().unwrap_or("*");
            println!("  {} {} => {}", "[PREVIEW]".cyan(), name.bold(), decl);
        }
        println!();
        return Ok(());
    }

    let mut installed = Vec::new();
    for spec in package_specs {
        let (name, version_opt) = parse_ruby_gem_spec(spec);
        let ver = gem_version_arg_from_spec(spec);

        let ok = if use_bundler {
            run_bundle_add(&name, ver.as_deref())
        } else {
            ruby_gems::gem_install(&name, ver.as_deref()).map(|_| ())
        };

        match ok {
            Ok(()) => {
                println!(
                    "  {} {}",
                    "[OK]".green(),
                    format!("Installed {}", spec.bold())
                );
                if !use_bundler {
                    if let Ok(mut gf) = Gemfile::load_or_default(&cwd) {
                        if gf.exists() {
                            gf.upsert(&name, version_opt.as_deref());
                            let _ = gf.write();
                        }
                    }
                }
                let declared = ruby_gems::gem_local_version(&name)?
                    .filter(|v| !v.is_empty())
                    .map(|v| format!(">={v}"))
                    .unwrap_or_else(|| "*".into());
                installed.push((name, declared));
            }
            Err(e) => println!("  {} {} — {}", "[ERROR]".red(), spec, e),
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

fn which_bundle() -> bool {
    let bundle = runtime_tool("ruby", "bundle");
    Command::new(&bundle)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_bundle_add(name: &str, version: Option<&str>) -> Result<()> {
    let bundle = runtime_tool("ruby", "bundle");
    let mut cmd = Command::new(&bundle);
    cmd.args(["add", name]);
    if let Some(v) = version {
        cmd.args(["--version", v]);
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("bundle add failed to start ({:?}): {e}", bundle))?;
    if !status.success() {
        anyhow::bail!("bundle add exit code {:?}", status.code());
    }
    Ok(())
}

/// Version string for `gem install -v`, or omit for latest (None).
fn gem_version_arg_from_spec(spec: &str) -> Option<String> {
    let (_, v_opt) = parse_ruby_gem_spec(spec);
    match v_opt.as_deref() {
        None | Some("*") | Some("latest") | Some("") => None,
        Some(v) => Some(v.to_string()),
    }
}

fn parse_ruby_gem_spec(spec: &str) -> (String, Option<String>) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !name.is_empty() && !version.is_empty() {
            return (name.to_string(), Some(version.to_string()));
        }
    }
    (spec.to_string(), None)
}

pub(super) fn cmd_add_deno(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (deno)".bold().cyan());
    println!("  {} {} item(s)", "[PLAN]".cyan(), package_specs.len());

    let cwd = std::env::current_dir()?;
    if dry_run {
        for spec in package_specs {
            match deno_imports::parse_spec(spec) {
                Ok((k, v)) => println!("  {} {} -> {}", "[PREVIEW]".cyan(), k.bold(), v),
                Err(e) => println!("  {} {}: {}", "[ERROR]".red(), spec, e),
            }
        }
        println!();
        return Ok(());
    }

    // Try `deno add` first (Deno >= 1.42).
    let used_deno = match deno_imports::try_deno_add(package_specs) {
        Ok(true) => {
            println!(
                "  {} Used `deno add` for {} item(s)",
                "[OK]".green(),
                package_specs.len()
            );
            true
        }
        _ => false,
    };

    let mut installed = Vec::new();
    if !used_deno {
        let mut manifest = DenoManifest::load_or_create(&cwd)?;
        for spec in package_specs {
            match deno_imports::parse_spec(spec) {
                Ok((key, target)) => {
                    manifest.upsert_import(&key, &target);
                    installed.push((key, target));
                }
                Err(e) => println!("  {} {}: {}", "[ERROR]".red(), spec, e),
            }
        }
        manifest.write()?;
        println!("  {} Updated {}", "[OK]".green(), manifest.path().display());
    } else {
        // Reflect the new entries in ven.toml as well.
        for spec in package_specs {
            if let Ok((key, target)) = deno_imports::parse_spec(spec) {
                installed.push((key, target));
            }
        }
    }

    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

pub(super) fn cmd_add_bun(package_specs: &[String], dry_run: bool) -> Result<()> {
    println!("\n{}", "ven add (bun)".bold().cyan());
    println!("  {} {} package(s)", "[PLAN]".cyan(), package_specs.len());
    if dry_run {
        println!(
            "  {} Dry run mode - no changes will be made",
            "[DRY-RUN]".yellow()
        );
        println!();
        for spec in package_specs {
            println!("  {} bun add {}", "[PREVIEW]".cyan(), spec.bold());
        }
        println!();
        return Ok(());
    }

    let bun_bin = runtime_tool("bun", "bun");
    let mut installed = Vec::new();
    for spec in package_specs {
        let status = Command::new(&bun_bin).args(["add", spec]).status();
        match status {
            Ok(s) if s.success() => {
                println!("  {} {}", "[OK]".green(), format!("Added {}", spec.bold()));
                let (name, declared) = parse_go_spec(spec);
                installed.push((name, declared));
            }
            Ok(_) => println!("  {} Failed to add {}", "[ERROR]".red(), spec),
            Err(e) => println!(
                "  {} Could not run bun at {:?}: {}",
                "[ERROR]".red(),
                bun_bin,
                e
            ),
        }
    }
    if !installed.is_empty() {
        update_ven_toml_packages(&installed)?;
    }
    println!();
    Ok(())
}

fn ensure_go_mod() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let go_mod = cwd.join("go.mod");
    if go_mod.is_file() {
        return Ok(());
    }
    let module_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("app")
        .to_string();
    let go_bin = runtime_tool("go", "go");
    let status = Command::new(&go_bin)
        .args(["mod", "init", &module_name])
        .status()?;
    if !status.success() {
        anyhow::bail!("Failed to initialize go.mod (go mod init {})", module_name);
    }
    Ok(())
}

fn parse_go_spec(spec: &str) -> (String, String) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !version.is_empty() {
            return (name.to_string(), format!("@{}", version));
        }
    }
    (spec.to_string(), "latest".to_string())
}

fn ensure_cargo_manifest() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cargo_toml = cwd.join("Cargo.toml");
    if cargo_toml.is_file() {
        return Ok(());
    }
    let name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("app");
    let cargo_bin = runtime_tool("rust", "cargo");
    let status = Command::new(&cargo_bin)
        .args(["init", "--name", name])
        .status()?;
    if !status.success() {
        anyhow::bail!(
            "Failed to initialize Cargo.toml (cargo init --name {})",
            name
        );
    }
    Ok(())
}

fn parse_rust_spec(spec: &str) -> (String, String) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !version.is_empty() {
            return (name.to_string(), format!("@{}", version));
        }
    }
    (spec.to_string(), "latest".to_string())
}

fn parse_python_spec(spec: &str) -> (String, String) {
    let ops = ["==", ">=", "<=", "!=", "~=", ">", "<"];
    for op in ops {
        if let Some((name, _)) = spec.split_once(op) {
            return (name.trim().to_string(), spec.trim().to_string());
        }
    }
    (spec.trim().to_string(), "*".to_string())
}

/// If the project's `ven.toml` declares a Python runtime but no `./venv` exists yet,
/// create one using the ven-managed interpreter. This makes `ven add <pkg>` install
/// into the project venv without requiring the user to activate it (or even run
/// `ven init` first).
fn ensure_project_python_venv() -> Result<()> {
    let cwd = std::env::current_dir()?;
    if local_venv_bin_dir(&cwd).is_some() {
        return Ok(()); // venv already there
    }
    use crate::core::project_venv::{create_local_venv, ensure_gitignore_venv, PROJECT_VENV_DIR};
    let python = runtime_tool("python", "python");
    if python == PathBuf::from("python") {
        // No ven-managed Python and no project venv. Nothing to do; pip install will
        // fall back to whatever python is on PATH (legacy behaviour).
        return Ok(());
    }
    println!(
        "  {} Creating local Python env at `{}/` ({})...",
        "[PY]".cyan().bold(),
        PROJECT_VENV_DIR,
        python.display()
    );
    match create_local_venv(&cwd, &python) {
        Ok(path) => {
            let _ = ensure_gitignore_venv(&cwd);
            println!(
                "  {} venv ready at {}",
                "[OK]".green().bold(),
                path.display()
            );
        }
        Err(e) => {
            println!(
                "  {} Could not auto-create venv: {} (continuing with the runtime's pip)",
                "[!]".yellow(),
                e
            );
        }
    }
    Ok(())
}

/// Prefer the project's local venv (`./venv/Scripts/python.exe`), then `VIRTUAL_ENV`,
/// then the ven-managed runtime from `ven.toml`, then bare `python`/`python3`.
///
/// This ordering ensures `ven add` always installs into the project's isolated
/// environment when `ven init` (for Python) has been run, even without manually
/// activating the venv first.
fn resolve_python_cmd() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(bin) = local_venv_bin_dir(&cwd) {
            #[cfg(target_os = "windows")]
            let exe = bin.join("python.exe");
            #[cfg(not(target_os = "windows"))]
            let exe = bin.join("python");
            if exe.is_file() {
                return exe;
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
            let p = PathBuf::from(venv).join("Scripts").join("python.exe");
            if p.is_file() {
                return p;
            }
        }
        let resolved = runtime_tool("python", "python");
        if resolved != PathBuf::from("python") {
            return resolved;
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
        if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
            let p = PathBuf::from(venv).join("bin").join("python");
            if p.is_file() {
                return p;
            }
        }
        let resolved = runtime_tool("python", "python3");
        if resolved != PathBuf::from("python3") {
            return resolved;
        }
        PathBuf::from("python3")
    }
}
