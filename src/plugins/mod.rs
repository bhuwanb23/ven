use anyhow::Result;
use std::path::PathBuf;

// Every language backend (Node, Python, Go...) implements this trait.
// It defines the 4 operations ven needs for any language.
#[allow(dead_code)]
pub trait LanguagePlugin: Send + Sync {
    fn name(&self) -> &str;

    // Install a specific version (delegates to backend tool)
    fn install_version(&self, version: &str) -> Result<()>;

    // List all installed versions
    fn list_installed(&self) -> Result<Vec<String>>;

    // Return the bin/ directory path for a given installed version
    // This path gets prepended to PATH during activation
    fn bin_path(&self, version: &str) -> Result<PathBuf>;

    // Return the latest available version from the internet
    fn latest_version(&self) -> Result<String>;
}

pub mod node;
pub mod registry;

pub use node::NodePlugin;
pub use registry::PluginRegistry;