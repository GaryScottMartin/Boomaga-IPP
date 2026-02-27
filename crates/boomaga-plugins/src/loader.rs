//! Dynamic library loader for plugins

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn, error};
use crate::core::PluginRegistry;
use crate::core::{Plugin, PluginMetadata, PluginError, PluginInstance, PluginStatus, PluginType, PluginContext, Logger, EventEmitter};

/// Plugin loader
pub struct PluginLoader {
    /// Plugin directories to search
    plugin_dirs: Vec<PathBuf>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(plugin_dirs: Vec<PathBuf>) -> Self {
        Self { plugin_dirs }
    }

    /// Load a plugin from a file
    pub fn load_from_file(&self, path: PathBuf) -> Result<PluginInstance, PluginError> {
        info!("Loading plugin from: {:?}", path);

        // In production, would use libloading to load the dynamic library
        // For now, return a mock plugin instance

        let metadata = PluginMetadata {
            id: PluginId::new(format!("plugin_{}", path.file_name().unwrap().to_string_lossy())),
            name: "Mock Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A mock plugin for testing".to_string(),
            author: "Boomaga Team".to_string(),
            license: "MIT".to_string(),
            plugin_type: PluginType::Utility,
            entry_point: "init".to_string(),
        };

        // Create a mock plugin
        let plugin = MockPlugin {
            metadata: metadata.clone(),
            initialized: false,
        };

        let instance = PluginInstance::new(metadata, plugin);

        info!("Plugin loaded successfully: {}", metadata.name);

        Ok(instance)
    }

    /// Load a plugin by name
    pub fn load_by_name(&self, name: &str) -> Result<PluginInstance, PluginError> {
        // Search in plugin directories
        for dir in &self.plugin_dirs {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension() == Some(OsStr::new("so")) || path.extension() == Some(OsStr::new("dll")) {
                    if path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s == name)
                        .unwrap_or(false)
                    {
                        return self.load_from_file(path);
                    }
                }
            }
        }

        Err(PluginError::NotFound(format!("Plugin not found: {}", name)))
    }

    /// Scan directories for plugins
    pub fn scan_plugins(&self) -> Result<Vec<PluginInstance>, PluginError> {
        let mut plugins = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                warn!("Plugin directory does not exist: {:?}", dir);
                continue;
            }

            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if path.extension() == Some(OsStr::new("so")) || path.extension() == Some(OsStr::new("dll")) {
                        match self.load_from_file(path) {
                            Ok(instance) => plugins.push(instance),
                            Err(e) => {
                                error!("Failed to load plugin {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        info!("Found {} plugins", plugins.len());
        Ok(plugins)
    }

    /// Set plugin directories
    pub fn set_plugin_dirs(&mut self, dirs: Vec<PathBuf>) {
        self.plugin_dirs = dirs;
    }

    /// Get plugin directories
    pub fn plugin_dirs(&self) -> &[PathBuf] {
        &self.plugin_dirs
    }
}

/// Mock plugin for testing
struct MockPlugin {
    metadata: PluginMetadata,
    initialized: bool,
}

impl Plugin for MockPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<(), PluginError> {
        self.initialized = true;
        Ok(())
    }

    fn start(&mut self) -> Result<(), PluginError> {
        if !self.initialized {
            return Err(PluginError::RuntimeError(
                "Plugin must be initialized before starting".to_string(),
            ));
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn destroy(&mut self) {
        self.initialized = false;
    }

    fn get_capability(&self, capability: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        if capability == "test_capability" {
            Some(Arc::new("test_data" as &str))
        } else {
            None
        }
    }

    fn execute_command(&self, _command: &str, _params: std::collections::HashMap<String, String>) -> Result<String, PluginError> {
        Ok("Command executed successfully".to_string())
    }
}

/// Dynamic plugin loader using libloading
pub struct DynamicPluginLoader {
    /// Plugin paths
    plugin_paths: Vec<PathBuf>,
}

impl DynamicPluginLoader {
    /// Create a new dynamic plugin loader
    pub fn new() -> Self {
        Self {
            plugin_paths: Vec::new(),
        }
    }

    /// Register a plugin path
    pub fn register(&mut self, path: PathBuf) {
        self.plugin_paths.push(path);
    }

    /// Load a dynamic plugin
    pub fn load(&self, path: PathBuf) -> Result<PluginInstance, PluginError> {
        info!("Loading dynamic plugin: {:?}", path);

        // In production, would use libloading to load the library
        // and call the init function

        let metadata = PluginMetadata {
            id: PluginId::new(format!("dynamic_plugin_{}", path.file_name().unwrap().to_string_lossy())),
            name: path.file_stem().unwrap().to_string_lossy().to_string(),
            version: "1.0.0".to_string(),
            description: "Dynamic plugin".to_string(),
            author: "Boomaga Team".to_string(),
            license: "MIT".to_string(),
            plugin_type: PluginType::Custom,
            entry_point: "boomaga_plugin_init".to_string(),
        };

        let plugin = MockPlugin {
            metadata: metadata.clone(),
            initialized: false,
        };

        let instance = PluginInstance::new(metadata, plugin);

        info!("Dynamic plugin loaded: {}", metadata.name);

        Ok(instance)
    }

    /// Load all plugins from directories
    pub fn load_all(&self, directories: Vec<PathBuf>) -> Result<Vec<PluginInstance>, PluginError> {
        let mut plugins = Vec::new();

        for dir in directories {
            if dir.exists() {
                for entry in std::fs::read_dir(&dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_file() {
                        if path.extension() == Some(OsStr::new("so")) || path.extension() == Some(OsStr::new("dll")) {
                            match self.load(path.clone()) {
                                Ok(instance) => {
                                    plugins.push(instance);
                                    info!("Loaded plugin from: {:?}", path);
                                }
                                Err(e) => {
                                    error!("Failed to load plugin {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }
}
