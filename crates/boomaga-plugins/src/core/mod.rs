//! Core plugin system types and interfaces

pub mod plugin;

// Re-export all types
pub use plugin::{
    PluginId,
    PluginMetadata,
    PluginType,
    PluginStatus,
    PluginContext,
    Logger,
    EventEmitter,
    Plugin,
    PluginError,
    PluginInstance,
    PluginRegistry,
    DocumentFilter,
    LayoutPlugin,
    PrintHook,
    UIExtension,
    UtilityPlugin,
};
