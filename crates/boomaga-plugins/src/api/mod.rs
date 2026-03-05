//! Plugin API definitions

use super::core::{Plugin, PluginMetadata, PluginType, PluginId, PluginRegistry, PluginInstance};
use super::core::{
    PluginError,
    DocumentFilter,
    LayoutPlugin,
    PrintHook,
    UIExtension,
    UtilityPlugin,
};
use boomaga_core::document::Document;
use boomaga_core::{PageSize, Page, PrintJobRequest, Error, Result};
use std::any::Any;
use std::sync::Arc;

/// Plugin capability registry
pub struct PluginCapabilityRegistry {
    /// Capabilities
    capabilities: std::collections::HashMap<String, Vec<PluginId>>,
}

impl PluginCapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: std::collections::HashMap::new(),
        }
    }

    /// Register capability
    pub fn register_capability(&mut self, capability: &str, plugin_id: PluginId) {
        self.capabilities
            .entry(capability.to_string())
            .or_insert_with(Vec::new)
            .push(plugin_id);
    }

    /// Get plugins with capability
    pub fn get_plugins_with_capability(&self, capability: &str) -> Vec<&PluginId> {
        self.capabilities
            .get(capability)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }

    /// Check if plugin has capability
    pub fn has_capability(&self, plugin_id: &PluginId, capability: &str) -> bool {
        self.capabilities
            .get(capability)
            .map(|ids| ids.contains(plugin_id))
            .unwrap_or(false)
    }
}

impl Default for PluginCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin API result
pub type PluginResult<T> = std::result::Result<T, crate::core::PluginError>;

/// Plugin manager
pub struct PluginManager {
    /// Registry
    registry: PluginRegistry,
    /// Capabilities
    capabilities: PluginCapabilityRegistry,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
            capabilities: PluginCapabilityRegistry::new(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: PluginInstance) {
        let metadata = plugin.metadata();
        let plugin_type_str = metadata.plugin_type.as_str().to_string();
        let id = metadata.id.clone();
        self.registry.register(plugin);
        self.capabilities.register_capability(
            &plugin_type_str,
            id,
        );
    }

    /// Initialize all plugins
    pub fn initialize_all(&mut self) -> Result<(), crate::core::PluginError> {
        self.registry.initialize_all()
    }

    /// Start all plugins
    pub fn start_all(&mut self) -> Result<(), crate::core::PluginError> {
        self.registry.start_all()
    }

    /// Stop all plugins
    pub fn stop_all(&mut self) -> Result<(), crate::core::PluginError> {
        self.registry.stop_all()
    }

    /// Destroy all plugins
    pub fn destroy_all(&mut self) {
        self.registry.destroy_all();
    }

    /// Get registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get capabilities
    pub fn capabilities(&self) -> &PluginCapabilityRegistry {
        &self.capabilities
    }

    /// Get plugins by type
    pub fn plugins_by_type(&self, plugin_type: PluginType) -> Vec<&PluginInstance> {
        self.registry.by_type(plugin_type)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
