use std::collections::HashMap;
use super::LanguagePlugin;
use super::NodePlugin;

/// Plugin Registry - manages all language plugins
/// This makes it easy to add new languages (Python, Go, Rust, etc.)
/// without modifying CLI code
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn LanguagePlugin>>,
}

impl PluginRegistry {
    /// Create a new registry with all available plugins registered
    pub fn new() -> Self {
        let mut registry = Self {
            plugins: HashMap::new(),
        };
        
        // Register all available language plugins
        registry.register("node", Box::new(NodePlugin));
        // Future: registry.register("python", Box::new(PythonPlugin));
        // Future: registry.register("go", Box::new(GoPlugin));
        // Future: registry.register("rust", Box::new(RustPlugin));
        
        registry
    }
    
    /// Register a new language plugin
    pub fn register(&mut self, name: &str, plugin: Box<dyn LanguagePlugin>) {
        self.plugins.insert(name.to_string(), plugin);
    }
    
    /// Get a plugin by language name
    pub fn get(&self, name: &str) -> Option<&dyn LanguagePlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
    
    /// Get a plugin by language name, or return an error if not found
    pub fn require(&self, name: &str) -> anyhow::Result<&dyn LanguagePlugin> {
        self.get(name)
            .ok_or_else(|| {
                let available = self.list_languages();
                anyhow::anyhow!(
                    "Unknown language: '{}'. Supported languages: {}",
                    name,
                    available.join(", ")
                )
            })
    }
    
    /// List all registered language names
    pub fn list_languages(&self) -> Vec<&str> {
        let mut languages: Vec<&str> = self.plugins.keys()
            .map(|s| s.as_str())
            .collect();
        languages.sort(); // Alphabetical order
        languages
    }
    
    /// Check if a language is supported
    pub fn is_supported(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
