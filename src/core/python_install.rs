//! Windows Python embeddable distribution install (~/.ven/python/<version>/).
//! https://www.python.org/downloads/windows/ — zip layout, `pythonXY._pth`, pip bootstrap.

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipArchive;

const GET_PIP_URL: &str = "https://bootstrap.pypa.io/get-pip.py";

pub struct PythonDownloader {
    storage_root: PathBuf,
    cache_dir: PathBuf,
}

impl PythonDownloader {
    pub fn new() -> Result<Self> {
        let storage_root = std::env::var("VEN_STORAGE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("Cannot find home directory")
                    .join(".ven")
            });
        let cache_dir = storage_root.join(".cache");
        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    #[cfg(target_os = "windows")]
    fn embed_arch_tag() -> Result<&'static str> {
        match std::env::consts::ARCH {
            "x86_64" => Ok("amd64"),
            "aarch64" => Ok("arm64"),
            "x86" => Ok("win32"),
            other => Err(anyhow!(
                "Unsupported Windows Python embed architecture: {}",
                other
            )),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn embed_arch_tag() -> Result<&'static str> {
        Err(anyhow!(
            "Python embed install is only implemented on Windows in this release."
        ))
    }

    /// https://www.python.org/ftp/python/3.12.7/python-3.12.7-embed-amd64.zip
    pub fn build_embed_zip_url(version: &str) -> Result<String> {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = version;
            Self::embed_arch_tag()?;
            unreachable!()
        }
        #[cfg(target_os = "windows")]
        {
            let ver = normalize_python_version(version)?;
            let arch = Self::embed_arch_tag()?;
            Ok(format!(
                "https://www.python.org/ftp/python/{ver}/python-{ver}-embed-{arch}.zip",
                ver = ver,
                arch = arch
            ))
        }
    }

    pub fn get_install_dir(&self, version: &str) -> PathBuf {
        let ver = version
            .trim()
            .trim_start_matches(|c: char| c == 'v' || c == 'V')
            .to_string();
        self.storage_root.join("python").join(ver)
    }

    pub fn get_bin_path(&self, version: &str) -> Result<PathBuf> {
        let dir = self.get_install_dir(version);
        if !dir.exists() {
            return Err(anyhow!(
                "Python {} is not installed. Run: ven install python {}",
                version,
                version
            ));
        }
        let exe = dir.join("python.exe");
        if exe.exists() {
            Ok(dir)
        } else {
            Err(anyhow!("Python {} not found at {}", version, dir.display()))
        }
    }

    pub fn list_installed(&self) -> Result<Vec<String>> {
        let py_dir = self.storage_root.join("python");
        if !py_dir.exists() {
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(&py_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let s = name.to_string_lossy();
                    if s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                        && s.contains('.')
                    {
                        versions.push(s.into_owned());
                    }
                }
            }
        }
        versions.sort_by(|a, b| version_cmp_parts(b, a));
        Ok(versions)
    }

    fn download_zip(&self, url: &str, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        println!("{} {}", "[FETCH]".cyan(), url);
        let client = Client::new();
        let bytes = client.get(url).send()?.error_for_status()?.bytes()?;
        let mut f = File::create(dest)?;
        f.write_all(&bytes)?;
        println!("{} Saved {}", "[OK]".green(), dest.display());
        Ok(())
    }

    fn extract_zip(zip_path: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest)?;
        let file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        println!("{} Extracting embeddable Python...", "[ARROW]".cyan());
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let outpath = dest.join(entry.mangled_name());
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            if entry.is_file() {
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut entry, &mut outfile)?;
            } else {
                fs::create_dir_all(&outpath)?;
            }
        }
        Ok(())
    }

    pub fn install_embedded(&self, version: &str) -> Result<()> {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = version;
            return Err(anyhow!(
                "ven install python uses the Windows embeddable zip only in this release.\n\
                 Use Linux/macOS system Python or pyenv for now."
            ));
        }

        #[cfg(target_os = "windows")]
        {
            let ver = normalize_python_version(version)?;
            let url = Self::build_embed_zip_url(&ver)?;
            let filename = url.split('/').last().unwrap_or("python-embed.zip");
            fs::create_dir_all(&self.cache_dir)?;
            let cache_zip = self.cache_dir.join(filename);

            if !cache_zip.exists() {
                self.download_zip(&url, &cache_zip)?;
            } else {
                println!("{} Using cached {}", "[OK]".green(), cache_zip.display());
            }

            let install_dir = self.get_install_dir(&ver);
            if install_dir.exists() {
                fs::remove_dir_all(&install_dir)
                    .with_context(|| format!("Could not remove {}", install_dir.display()))?;
            }
            fs::create_dir_all(&install_dir)?;
            Self::extract_zip(&cache_zip, &install_dir)?;

            enable_embed_import_site(&install_dir)?;
            bootstrap_pip(&install_dir)?;
            validate_python_pip(&install_dir, &ver)?;
            Ok(())
        }
    }
}

fn version_cmp_parts(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u32> = a.split('.').filter_map(|x| x.parse().ok()).collect();
    let pb: Vec<u32> = b.split('.').filter_map(|x| x.parse().ok()).collect();
    pa.cmp(&pb)
}

/// Normalize to `major.minor.patch` (no `v` prefix).
pub fn normalize_python_version(version: &str) -> Result<String> {
    let v = version.trim().trim_start_matches(['v', 'V']);
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() >= 3 && parts.iter().all(|p| !p.is_empty()) {
        return Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]));
    }
    Err(anyhow!(
        "Need full Python version for embeddable zip, e.g. 3.12.7 (got '{}')",
        version
    ))
}

fn find_pth_file(install_dir: &Path) -> Result<PathBuf> {
    for entry in fs::read_dir(install_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("_pth") {
            return Ok(path);
        }
    }
    Err(anyhow!(
        "No python*. _pth file found under {}",
        install_dir.display()
    ))
}

/// Uncomment `# import site` so pip / site-packages work.
fn enable_embed_import_site(install_dir: &Path) -> Result<()> {
    let pth = find_pth_file(install_dir)?;
    let mut content =
        fs::read_to_string(&pth).with_context(|| format!("Read {}", pth.display()))?;
    // Typical embed line: `# import site`
    if content.contains("# import site") {
        content = content.replace("# import site", "import site");
    } else if content.contains("#import site") {
        content = content.replace("#import site", "import site");
    } else if !content.lines().any(|l| l.trim() == "import site") {
        content.push_str("\nimport site\n");
    }
    fs::write(&pth, content)?;
    println!(
        "{} Enabled import site in {}",
        "[OK]".green(),
        pth.display()
    );
    Ok(())
}

fn bootstrap_pip(install_dir: &Path) -> Result<()> {
    let python_exe = install_dir.join("python.exe");
    println!("{} Bootstrapping pip (ensurepip)...", "[PIP]".cyan());
    let st = Command::new(&python_exe)
        .current_dir(install_dir)
        .args(["-m", "ensurepip", "--upgrade"])
        .status();

    let ok = matches!(st, Ok(s) if s.success());
    if ok {
        println!("{} pip installed via ensurepip", "[OK]".green());
        return Ok(());
    }

    println!(
        "{} ensurepip failed or unavailable; trying get-pip.py",
        "!".yellow()
    );
    let client = Client::new();
    let script = client.get(GET_PIP_URL).send()?.error_for_status()?.text()?;
    let gp = install_dir.join("get-pip.py");
    fs::write(&gp, script)?;

    let st2 = Command::new(&python_exe)
        .current_dir(install_dir)
        .arg(&gp)
        .status()
        .with_context(|| "Could not run python get-pip.py")?;
    if !st2.success() {
        return Err(anyhow!("get-pip.py exited with status {:?}", st2.code()));
    }
    println!("{} pip installed via get-pip.py", "[OK]".green());
    Ok(())
}

fn validate_python_pip(install_dir: &Path, version: &str) -> Result<()> {
    let python_exe = install_dir.join("python.exe");
    let py_out = Command::new(&python_exe)
        .current_dir(install_dir)
        .args(["--version"])
        .output()
        .with_context(|| "python --version")?;
    if !py_out.status.success() {
        return Err(anyhow!(
            "python --version failed: {}",
            String::from_utf8_lossy(&py_out.stderr)
        ));
    }
    print!(
        "{} {}",
        "[OK]".green(),
        String::from_utf8_lossy(&py_out.stdout).trim()
    );

    let pip_out = Command::new(&python_exe)
        .current_dir(install_dir)
        .args(["-m", "pip", "--version"])
        .output()
        .with_context(|| "python -m pip --version")?;
    if !pip_out.status.success() {
        return Err(anyhow!(
            "pip check failed: {}",
            String::from_utf8_lossy(&pip_out.stderr)
        ));
    }
    println!(
        "\n{} {}",
        "[OK]".green(),
        String::from_utf8_lossy(&pip_out.stdout).trim()
    );
    println!(
        "\n{} Python {} ready at {}",
        "✓".green(),
        version.bold(),
        install_dir.display()
    );
    Ok(())
}

/// Versions listed at https://www.python.org/ftp/python/
pub fn fetch_python_release_versions() -> Result<Vec<String>> {
    let html = Client::new()
        .get("https://www.python.org/ftp/python/")
        .send()?
        .error_for_status()?
        .text()?;
    let mut out = Vec::new();
    for part in html.split("href=\"").skip(1) {
        if let Some(end) = part.find('/') {
            let seg = &part[..end];
            if seg.chars().all(|c| c.is_ascii_digit() || c == '.') && seg.matches('.').count() >= 2
            {
                out.push(seg.to_string());
            }
        }
    }
    out.sort_by(|a, b| version_cmp_parts(b, a));
    Ok(out)
}

/// Latest `major.minor.patch` matching `3` / `3.12` / exact triple.
pub fn resolve_python_version_spec(spec: &str, available: &[String]) -> Result<String> {
    let spec = spec.trim();
    if spec.eq_ignore_ascii_case("latest") || spec.eq_ignore_ascii_case("lts") {
        return available
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No Python versions listed on python.org"));
    }

    let exact: Vec<_> = spec.split('.').filter(|s| !s.is_empty()).collect();
    if exact.len() >= 3 {
        let v = format!("{}.{}.{}", exact[0], exact[1], exact[2]);
        if available.iter().any(|a| a == &v) {
            return Ok(v);
        }
        return Err(anyhow!(
            "Python {} not found on python.org ftp listing (try another patch).",
            v
        ));
    }

    if exact.len() == 2 {
        let prefix = format!("{}.", format!("{}.{}", exact[0], exact[1]));
        let mut hits: Vec<&String> = available
            .iter()
            .filter(|a| a.starts_with(&prefix))
            .collect();
        hits.sort_by(|a, b| version_cmp_parts(b, a));
        return hits
            .first()
            .map(|s| (*s).clone())
            .ok_or_else(|| anyhow!("No Python {}.x.y release found", exact[0]));
    }

    if exact.len() == 1 {
        let prefix = format!("{}.", exact[0]);
        let mut hits: Vec<&String> = available
            .iter()
            .filter(|a| a.starts_with(&prefix))
            .collect();
        hits.sort_by(|a, b| version_cmp_parts(b, a));
        return hits
            .first()
            .map(|s| (*s).clone())
            .ok_or_else(|| anyhow!("No Python {}.x.y release found", exact[0]));
    }

    Err(anyhow!("Invalid Python version spec: {}", spec))
}

pub fn install_python(downloader: &PythonDownloader, version: &str) -> Result<()> {
    downloader.install_embedded(version)
}
