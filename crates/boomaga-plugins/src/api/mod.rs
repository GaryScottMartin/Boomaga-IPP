//! Plugin API definitions

use crate::core::{Plugin, PluginMetadata, PluginType};
use std::any::Any;
use std::sync::Arc;

/// Document filter API
pub trait DocumentFilter: Plugin {
    /// Filter a document
    fn filter_document(&mut self, document: &mut crate::document::Document) -> Result<(), crate::Error>;

    /// Get supported file formats
    fn supported_formats(&self) -> Vec<String>;
}

/// Layout API
pub trait LayoutPlugin: Plugin {
    /// Generate layout
    fn generate_layout(
        &self,
        pages: &[crate::core::Page],
        output_size: crate::PageSize,
    ) -> Result<Vec<crate::core::Page>, crate::Error>;
}

/// Print hook API
pub trait PrintHook: Plugin {
    /// Before print
    fn before_print(&self, job: &crate::PrintJobRequest) -> Result<(), crate::Error>;

    /// After print
    fn after_print(&self, job: &crate::PrintJobRequest, success: bool) -> Result<(), crate::Error>;
}

/// UI extension API
pub trait UIExtension: Plugin {
    /// Add menu item
    fn add_menu_item(&self, menu_name: &str, item_name: &str) -> Result<(), crate::Error>;

    /// Add toolbar button
    fn add_toolbar_button(&self, button_name: &str) -> Result<(), crate::Error>;

    /// Add shortcut
    fn add_shortcut(&self, shortcut: &str, command: &str) -> Result<(), crate::Error>;
}

/// Utility API
pub trait UtilityPlugin: Plugin {
    /// Execute utility function
    fn execute(&self, command: &str, params: std::collections::HashMap<String, String>) -> Result<String, crate::Error>;

    /// Get utility capabilities
    fn capabilities(&self) -> Vec<String>;
}

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
        self.registry.register(plugin);
        self.capabilities.register_capability(
            plugin.metadata().plugin_type.as_str(),
            plugin.metadata().id.clone(),
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
