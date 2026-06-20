use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    use std::fs;

    fn make_installer(tmp: &std::path::Path) -> BaseInstaller {
        let storage_root = tmp.to_path_buf();
        let cache_dir = storage_root.join(".cache");
        BaseInstaller {
            storage_root,
            cache_dir,
        }
    }

    #[test]
    fn new_creates_valid_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        std::env::set_var("VEN_HOME", home.to_str().unwrap());
        std::env::set_var("VEN_STORAGE_PATH", "");

        let installer = BaseInstaller::new().unwrap();
        let sr = installer.storage_root.to_string_lossy();
        assert!(sr.ends_with(".ven") || sr == home.to_str().unwrap());
        let cd = installer.cache_dir.to_string_lossy();
        assert!(cd.ends_with(".cache"));

        std::env::remove_var("VEN_HOME");
        std::env::remove_var("VEN_STORAGE_PATH");
    }

    #[test]
    fn get_install_dir_correct_format() {
        let tmp = tempfile::tempdir().unwrap();
        let installer = make_installer(tmp.path());

        let dir = installer.get_install_dir("python", "3.11.4");
        assert_eq!(dir, tmp.path().join("python").join("3.11.4"));
    }

    #[test]
    fn list_installed_empty_for_nonexistent_language() {
        let tmp = tempfile::tempdir().unwrap();
        let installer = make_installer(tmp.path());

        let versions = installer.list_installed("nonexistent").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn list_installed_returns_sorted_newest_first() {
        let tmp = tempfile::tempdir().unwrap();
        let installer = make_installer(tmp.path());

        let lang_dir = tmp.path().join("python");
        fs::create_dir_all(&lang_dir).unwrap();
        for v in ["3.9.0", "3.11.4", "3.10.2", "3.12.1"] {
            fs::create_dir(lang_dir.join(v)).unwrap();
        }

        let versions = installer.list_installed("python").unwrap();
        assert_eq!(versions, vec!["3.12.1", "3.11.4", "3.10.2", "3.9.0"]);
    }

    #[test]
    fn list_installed_skips_non_version_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let installer = make_installer(tmp.path());

        let lang_dir = tmp.path().join("node");
        fs::create_dir_all(&lang_dir).unwrap();
        fs::create_dir(lang_dir.join("20.11.0")).unwrap();
        fs::create_dir(lang_dir.join("lts")).unwrap();
        fs::create_dir(lang_dir.join("src")).unwrap();
        fs::create_dir(lang_dir.join("18.0.0")).unwrap();

        let versions = installer.list_installed("node").unwrap();
        assert_eq!(versions, vec!["20.11.0", "18.0.0"]);
    }

    #[test]
    fn version_cmp_parts_ordering() {
        assert_eq!(version_cmp_parts("20.11.0", "18.0.0"), Ordering::Greater);
        assert_eq!(version_cmp_parts("1.2.3", "1.2.4"), Ordering::Less);
        assert_eq!(version_cmp_parts("3.10.0", "3.10.0"), Ordering::Equal);
        assert_eq!(version_cmp_parts("1.10.0", "1.9.0"), Ordering::Greater);
        assert_eq!(version_cmp_parts("2.0.0-rc1", "2.0.0"), Ordering::Less);
    }
}
