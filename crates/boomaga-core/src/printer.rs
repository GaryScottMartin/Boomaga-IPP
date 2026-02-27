//! Printer information and capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a printer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub name: String,
    pub description: String,
    pub uri: String,
    pub is_remote: bool,
    pub status: PrinterStatus,
    pub capabilities: PrinterCapabilities,
    pub default_settings: PrintOptions,
    pub attributes: HashMap<String, String>,
}

/// Status of a printer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrinterStatus {
    /// Printer is idle and ready
    Idle,
    /// Printer is busy
    Busy,
    /// Printer is paused
    Paused,
    /// Printer is stopped
    Stopped,
    /// Printer is in error state
    Error,
    /// Printer is offline
    Offline,
}

impl std::fmt::Display for PrinterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrinterStatus::Idle => write!(f, "Idle"),
            PrinterStatus::Busy => write!(f, "Busy"),
            PrinterStatus::Paused => write!(f, "Paused"),
            PrinterStatus::Stopped => write!(f, "Stopped"),
            PrinterStatus::Error => write!(f, "Error"),
            PrinterStatus::Offline => write!(f, "Offline"),
        }
    }
}

/// Printer capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterCapabilities {
    pub supports_color: bool,
    pub supports_duplex: bool,
    pub supports_multiple_copies: bool,
    pub supports_collating: bool,
    pub supports_pages_per_sheet: bool,
    pub supported_duplex_modes: Vec<DuplexMode>,
    pub supported_page_sizes: Vec<PageSize>,
    pub supported_orientations: Vec<Orientation>,
    pub supported_margins: Vec<MarginMode>,
    pub supported_languages: Vec<String>,
}

impl Default for PrinterCapabilities {
    fn default() -> Self {
        Self {
            supports_color: false,
            supports_duplex: false,
            supports_multiple_copies: true,
            supports_collating: true,
            supports_pages_per_sheet: true,
            supported_duplex_modes: vec![DuplexMode::None],
            supported_page_sizes: vec![PageSize::A4],
            supported_orientations: vec![Orientation::Portrait],
            supported_margins: vec![MarginMode::Normal],
            supported_languages: vec!["C".to_string()],
        }
    }
}

/// Supported page layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageLayout {
    /// Single page
    Single,
    /// Two pages (booklet)
    Booklet,
    /// Multiple pages per sheet
    NUp { count: PagesPerSheet },
    /// Custom layout
    Custom { width: f64, height: f64 },
}

/// Supported paper sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperSize {
    /// Letter (8.5 x 11 inches)
    Letter,
    /// Legal (8.5 x 14 inches)
    Legal,
    /// A4 (210 x 297 mm)
    A4,
    /// A3 (297 x 420 mm)
    A3,
    /// A5 (148 x 210 mm)
    A5,
    /// Custom { width, height }
    Custom { width: f64, height: f64 },
}
