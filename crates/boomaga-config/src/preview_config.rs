//! Preview application configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Preview application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    /// Default window size
    pub default_window_size: (u32, u32),

    /// Enable hardware acceleration
    pub hardware_acceleration: bool,

    /// Default zoom level
    pub default_zoom: f64,

    /// Enable auto-zoom
    pub auto_zoom: bool,

    /// Auto-zoom threshold
    pub auto_zoom_threshold: f64,

    /// Default page layout
    pub default_page_layout: PageLayout,

    /// Enable smooth scrolling
    pub smooth_scrolling: bool,

    /// Enable smooth rendering
    pub smooth_rendering: bool,

    /// Maximum cache size (in MB)
    pub max_cache_size: u64,

    /// Enable document cache
    pub enable_cache: bool,

    /// Cache file directory
    pub cache_dir: PathBuf,

    /// Enable plugins
    pub enable_plugins: bool,

    /// Plugin search directories
    pub plugin_dirs: Vec<PathBuf>,

    /// Default print settings
    pub default_print_settings: PrintSettings,

    /// Enable shortcuts
    pub enable_shortcuts: bool,

    /// Keybindings
    pub keybindings: Keybindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    /// Default copies
    pub copies: u32,

    /// Default collate
    pub collate: bool,

    /// Default duplex mode
    pub duplex: DuplexMode,

    /// Default orientation
    pub orientation: Orientation,

    /// Default pages per sheet
    pub pages_per_sheet: u8,

    /// Default margins
    pub margins: MarginMode,

    /// Default scale
    pub scale: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PageLayout {
    /// Single page
    Single,
    /// Two pages (booklet)
    Booklet,
    /// N-up layout
    NUp { count: u8 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DuplexMode {
    /// No duplex
    None,
    /// Long edge binding
    LongEdge,
    /// Short edge binding
    ShortEdge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    /// Navigation keys
    pub navigation: KeybindingConfig,

    /// Zoom controls
    pub zoom: KeybindingConfig,

    /// Print controls
    pub print: KeybindingConfig,

    /// View controls
    pub view: KeybindingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    /// Next page
    pub next_page: String,

    /// Previous page
    pub prev_page: String,

    /// First page
    pub first_page: String,

    /// Last page
    pub last_page: String,

    /// Zoom in
    pub zoom_in: String,

    /// Zoom out
    pub zoom_out: String,

    /// Fit to page
    pub fit_page: String,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            default_window_size: (1200, 800),
            hardware_acceleration: true,
            default_zoom: 1.0,
            auto_zoom: true,
            auto_zoom_threshold: 0.95,
            default_page_layout: PageLayout::Single,
            smooth_scrolling: true,
            smooth_rendering: true,
            max_cache_size: 256,
            enable_cache: true,
            cache_dir: PathBuf::from(".cache/boomaga/pages"),
            enable_plugins: true,
            plugin_dirs: vec![
                PathBuf::from("~/.local/share/boomaga/plugins").into(),
                PathBuf::from("/usr/lib/boomaga/plugins").into(),
            ],
            default_print_settings: PrintSettings::default(),
            enable_shortcuts: true,
            keybindings: Keybindings::default(),
        }
    }
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            copies: 1,
            collate: false,
            duplex: DuplexMode::None,
            orientation: Orientation::Portrait,
            pages_per_sheet: 1,
            margins: MarginMode::Normal,
            scale: 1.0,
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            navigation: KeybindingConfig {
                next_page: "Ctrl+Right".to_string(),
                prev_page: "Ctrl+Left".to_string(),
                first_page: "Ctrl+Home".to_string(),
                last_page: "Ctrl+End".to_string(),
                zoom_in: "".to_string(),
                zoom_out: "".to_string(),
                fit_page: "".to_string(),
            },
            zoom: KeybindingConfig {
                zoom_in: "Ctrl++".to_string(),
                zoom_out: "Ctrl+-".to_string(),
                next_page: "".to_string(),
                prev_page: "".to_string(),
                first_page: "".to_string(),
                last_page: "".to_string(),
                fit_page: "Ctrl+0".to_string(),
            },
            print: KeybindingConfig {
                zoom_in: "".to_string(),
                zoom_out: "".to_string(),
                next_page: "".to_string(),
                prev_page: "".to_string(),
                first_page: "".to_string(),
                last_page: "".to_string(),
                fit_page: "Ctrl+P".to_string(),
            },
            view: KeybindingConfig {
                zoom_in: "Ctrl+F".to_string(),
                zoom_out: "".to_string(),
                next_page: "F".to_string(),
                prev_page: "".to_string(),
                first_page: "Home".to_string(),
                last_page: "End".to_string(),
                fit_page: "Escape".to_string(),
            },
        }
    }
}

impl PreviewConfig {
    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.default_window_size.0 < 800 {
            return Err(anyhow::anyhow!("Window width must be at least 800"));
        }

        if self.default_window_size.1 < 600 {
            return Err(anyhow::anyhow!("Window height must be at least 600"));
        }

        if self.default_zoom <= 0.0 || self.default_zoom > 5.0 {
            return Err(anyhow::anyhow!("Default zoom must be between 0 and 5"));
        }

        if self.max_cache_size == 0 {
            return Err(anyhow::anyhow!("Max cache size must be greater than 0"));
        }

        Ok(())
    }

    /// Enable hardware acceleration
    pub fn with_hardware_acceleration(mut self, enabled: bool) -> Self {
        self.hardware_acceleration = enabled;
        self
    }

    /// Set default window size
    pub fn with_window_size(mut self, size: (u32, u32)) -> Self {
        self.default_window_size = size;
        self
    }

    /// Enable plugins
    pub fn with_plugins(mut self, enabled: bool) -> Self {
        self.enable_plugins = enabled;
        self
    }
}

impl From<PreviewConfig> for boomaga_core::constants::AppConfig {
    fn from(config: PreviewConfig) -> Self {
        boomaga_core::constants::AppConfig {
            preview_window: std::path::PathBuf::from(format!(
                "{}x{}",
                config.default_window_size.0, config.default_window_size.1
            )),
        }
    }
}
