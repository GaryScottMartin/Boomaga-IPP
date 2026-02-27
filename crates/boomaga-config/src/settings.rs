//! User settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Window settings
    pub window: WindowSettings,

    /// Document settings
    pub document: DocumentSettings,

    /// Print settings
    pub print: PrintSettings,

    /// UI settings
    pub ui: UISettings,

    /// Performance settings
    pub performance: PerformanceSettings,

    /// Plugin settings
    pub plugins: PluginSettings,

    /// Keybindings
    pub keybindings: HashMap<String, String>,
}

/// Window settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    /// Window position
    pub position: Option<(i32, i32)>,

    /// Window size
    pub size: Option<(u32, u32)>,

    /// Window maximized
    pub maximized: bool,

    /// Window fullscreen
    pub fullscreen: bool,

    /// Remember window state
    pub remember_state: bool,
}

/// Document settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSettings {
    /// Open last document
    pub open_last_document: bool,

    /// Last document path
    pub last_document_path: Option<String>,

    /// Page number to open at
    pub default_page: usize,

    /// Zoom level
    pub zoom_level: f64,

    /// Zoom mode
    pub zoom_mode: ZoomMode,
}

/// Print settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    /// Last used printer
    pub last_printer: Option<String>,

    /// Last used settings
    pub last_print_settings: Option<PrintOptions>,
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    /// Dark mode enabled
    pub dark_mode: bool,

    /// Show toolbar
    pub show_toolbar: bool,

    /// Show menu bar
    pub show_menu_bar: bool,

    /// Show status bar
    pub show_status_bar: bool,

    /// Show page numbers
    pub show_page_numbers: bool,

    /// Show bookmarks
    pub show_bookmarks: bool,

    /// Show page thumbnails
    pub show_page_thumbnails: bool,

    /// Compact mode
    pub compact_mode: bool,
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable lazy loading
    pub lazy_loading: bool,

    /// Enable page caching
    pub page_caching: bool,

    /// Cache page thumbnails
    pub cache_thumbnails: bool,

    /// Thumbnail size (in pixels)
    pub thumbnail_size: usize,

    /// Enable hardware acceleration
    pub hardware_acceleration: bool,

    /// Render quality
    pub render_quality: RenderQuality,
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    /// Enabled plugins
    pub enabled_plugins: Vec<String>,

    /// Plugin configuration
    pub plugin_config: HashMap<String, PluginMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoomMode {
    /// Fit page
    Fit,

    /// Fit width
    FitWidth,

    /// Fit height
    FitHeight,

    /// Actual size
    ActualSize,

    /// Custom
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RenderQuality {
    /// Low quality
    Low,

    /// Medium quality
    Medium,

    /// High quality
    High,

    /// Ultra quality
    Ultra,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window: WindowSettings::default(),
            document: DocumentSettings::default(),
            print: PrintSettings::default(),
            ui: UISettings::default(),
            performance: PerformanceSettings::default(),
            plugins: PluginSettings::default(),
            keybindings: HashMap::new(),
        }
    }
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            position: None,
            size: Some((1200, 800)),
            maximized: false,
            fullscreen: false,
            remember_state: true,
        }
    }
}

impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            open_last_document: true,
            last_document_path: None,
            default_page: 1,
            zoom_level: 1.0,
            zoom_mode: ZoomMode::Fit,
        }
    }
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            show_toolbar: true,
            show_menu_bar: true,
            show_status_bar: true,
            show_page_numbers: true,
            show_bookmarks: true,
            show_page_thumbnails: false,
            compact_mode: false,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            lazy_loading: true,
            page_caching: true,
            cache_thumbnails: true,
            thumbnail_size: 120,
            hardware_acceleration: true,
            render_quality: RenderQuality::High,
        }
    }
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled_plugins: vec![],
            plugin_config: HashMap::new(),
        }
    }
}
