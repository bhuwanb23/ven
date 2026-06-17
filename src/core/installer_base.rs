use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct BaseInstaller {
    pub storage_root: PathBuf,
    pub cache_dir: PathBuf,
}

impl BaseInstaller {
    pub fn new() -> Result<Self> {
        let storage_root = crate::core::ven_home::ven_home();
        let cache_dir = storage_root.join(".cache");
        Ok(Self {
            storage_root,
            cache_dir,
        })
    }

    pub fn get_install_dir(&self, language: &str, version: &str) -> PathBuf {
        self.storage_root.join(language).join(version)
    }

    pub fn list_installed(&self, language: &str) -> Result<Vec<String>> {
        let lang_dir = self.storage_root.join(language);
        if !lang_dir.exists() {
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        for entry in std::fs::read_dir(&lang_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        versions.push(name_str.to_string());
                    }
                }
            }
        }
        versions.sort_by(|a, b| version_cmp_parts(a, b).reverse());
        Ok(versions)
    }
}

pub fn version_cmp_parts(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split(|c: char| c == '.' || c == '-' || c == '+')
            .filter_map(|n| n.parse().ok())
            .collect()
    };
    parse(a).cmp(&parse(b))
}
