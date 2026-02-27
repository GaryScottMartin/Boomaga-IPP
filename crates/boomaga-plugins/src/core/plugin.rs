//! Core plugin types and interfaces

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Plugin ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(String);

impl PluginId {
    /// Create a new plugin ID from string
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Create a new plugin ID
    pub fn from_uuid(uuid: std::uuid::Uuid) -> Self {
        Self(uuid.to_string())
    }

    /// Get the ID as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: PluginId,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Author
    pub author: String,
    /// License
    pub license: String,
    /// Plugin type
    pub plugin_type: PluginType,
    /// Entry point
    pub entry_point: String,
}

/// Plugin type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// Document filter plugin
    DocumentFilter,
    /// Layout plugin
    Layout,
    /// Print hook plugin
    PrintHook,
    /// UI extension plugin
    UIExtension,
    /// Utility plugin
    Utility,
    /// Custom plugin
    Custom,
}

/// Plugin status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is loaded
    Loaded,
    /// Plugin is enabled
    Enabled,
    /// Plugin is disabled
    Disabled,
    /// Plugin is in error state
    Error,
}

/// Plugin context
pub struct PluginContext {
    /// Plugin ID
    pub id: PluginId,
    /// Application configuration
    pub config: HashMap<String, String>,
    /// Logger
    pub logger: tracing::Logger,
    /// Event emitter
    pub events: EventEmitter,
}

/// Logger wrapper
pub struct Logger {
    /// Underlying logger
    logger: tracing_subscriber::Registry,
}

/// Event emitter
pub struct EventEmitter {
    /// Event handlers
    handlers: Vec<Box<dyn Fn(String, String) + Send + Sync>>,
}

/// Plugin trait
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError>;

    /// Start the plugin
    fn start(&mut self) -> Result<(), PluginError>;

    /// Stop the plugin
    fn stop(&mut self) -> Result<(), PluginError>;

    /// Destroy the plugin
    fn destroy(&mut self);

    /// Get a capability
    fn get_capability(&self, capability: &str) -> Option<Arc<dyn Any + Send + Sync>>;

    /// Execute a command
    fn execute_command(&self, command: &str, params: HashMap<String, String>) -> Result<String, PluginError>;
}

/// Plugin error
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Initialization error: {0}")]
    InitError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Capability error: {0}")]
    CapabilityError(String),

    #[error("Command error: {0}")]
    CommandError(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),
}

/// Plugin instance
pub struct PluginInstance {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin implementation
    plugin: Box<dyn Plugin>,
    /// Status
    status: PluginStatus,
    /// Error
    error: Option<String>,
}

impl PluginInstance {
    /// Create a new plugin instance
    pub fn new<M: Plugin + 'static>(metadata: PluginMetadata, plugin: M) -> Self {
        Self {
            metadata,
            plugin: Box::new(plugin),
            status: PluginStatus::Loaded,
            error: None,
        }
    }

    /// Initialize the plugin
    pub fn initialize(&mut self) -> Result<(), PluginError> {
        self.status = PluginStatus::Error;

        let context = PluginContext {
            id: self.metadata.id.clone(),
            config: HashMap::new(),
            logger: Logger::default(),
            events: EventEmitter::default(),
        };

        self.plugin.initialize(&context)?;
        self.status = PluginStatus::Enabled;
        Ok(())
    }

    /// Start the plugin
    pub fn start(&mut self) -> Result<(), PluginError> {
        if self.status != PluginStatus::Enabled {
            return Err(PluginError::RuntimeError(
                "Plugin must be initialized before starting".to_string(),
            ));
        }

        self.plugin.start()?;
        self.status = PluginStatus::Enabled;
        Ok(())
    }

    /// Stop the plugin
    pub fn stop(&mut self) -> Result<(), PluginError> {
        self.plugin.stop()?;
        Ok(())
    }

    /// Destroy the plugin
    pub fn destroy(&mut self) {
        self.plugin.destroy();
        self.status = PluginStatus::Disabled;
    }

    /// Get plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get plugin status
    pub fn status(&self) -> PluginStatus {
        self.status
    }

    /// Set error
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.status = PluginStatus::Error;
    }

    /// Get plugin capability
    pub fn get_capability(&self, capability: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.plugin.get_capability(capability)
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            logger: tracing_subscriber::Registry::default(),
        }
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl EventEmitter {
    /// Register an event handler
    pub fn on(&mut self, handler: impl Fn(String, String) + Send + Sync + 'static) {
        self.handlers.push(Box::new(handler));
    }

    /// Emit an event
    pub fn emit(&self, event_type: String, data: String) {
        for handler in &self.handlers {
            handler(event_type, data);
        }
    }
}

/// Plugin registry
pub struct PluginRegistry {
    /// Registered plugins
    plugins: HashMap<PluginId, PluginInstance>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, instance: PluginInstance) {
        let id = instance.metadata().id.clone();
        self.plugins.insert(id, instance);
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, id: &PluginId) {
        self.plugins.remove(id);
    }

    /// Get a plugin
    pub fn get(&self, id: &PluginId) -> Option<&PluginInstance> {
        self.plugins.get(id)
    }

    /// Get a mutable plugin
    pub fn get_mut(&mut self, id: &PluginId) -> Option<&mut PluginInstance> {
        self.plugins.get_mut(id)
    }

    /// List all plugins
    pub fn list(&self) -> Vec<&PluginInstance> {
        self.plugins.values().collect()
    }

    /// Initialize all plugins
    pub fn initialize_all(&mut self) -> Result<(), PluginError> {
        for instance in self.plugins.values_mut() {
            if instance.status() != PluginStatus::Enabled {
                instance.initialize()?;
                instance.start()?;
            }
        }
        Ok(())
    }

    /// Stop all plugins
    pub fn stop_all(&mut self) -> Result<(), PluginError> {
        for instance in self.plugins.values_mut() {
            instance.stop()?;
        }
        Ok(())
    }

    /// Destroy all plugins
    pub fn destroy_all(&mut self) {
        for instance in self.plugins.values_mut() {
            instance.destroy();
        }
    }

    /// Get plugins by type
    pub fn by_type(&self, plugin_type: PluginType) -> Vec<&PluginInstance> {
        self.plugins
            .values()
            .filter(|p| p.metadata().plugin_type == plugin_type)
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
