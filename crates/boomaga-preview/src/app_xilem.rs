//! Xilem-compatible application state

use std::path::PathBuf;
use boomaga_core::{Document, FileType};
use druid::kurbo::Rect;

/// Application data for Xilem
#[derive(Clone)]
pub struct AppData {
    /// Path to the current document
    pub document_path: Option<PathBuf>,
    /// Loaded document
    pub document: Option<Document>,
    /// Current page number (0-indexed)
    pub current_page: usize,
    /// Zoom level
    pub zoom: f64,
    /// Page margins
    pub margins: boomaga_core::MarginMode,
    /// Pages per sheet
    pub pages_per_sheet: boomaga_core::PagesPerSheet,
    /// Duplex mode
    pub duplex_mode: boomaga_core::DuplexMode,
    /// Print options
    pub print_options: boomaga_core::PrintOptions,
    /// Job history
    pub job_history: Vec<boomaga_core::JobId>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            document_path: None,
            document: None,
            current_page: 0,
            zoom: 1.0,
            margins: boomaga_core::MarginMode::Normal,
            pages_per_sheet: boomaga_core::PagesPerSheet::One,
            duplex_mode: boomaga_core::DuplexMode::None,
            print_options: boomaga_core::PrintOptions::default(),
            job_history: Vec::new(),
        }
    }
}

impl AppData {
    /// Navigate to previous page
    pub fn previous_page(&mut self) {
        if let Some(document) = &self.document {
            if self.current_page > 0 {
                self.current_page = self.current_page.saturating_sub(1);
            }
        }
    }

    /// Navigate to next page
    pub fn next_page(&mut self) {
        if let Some(document) = &self.document {
            if self.current_page < document.page_count() {
                self.current_page = (self.current_page + 1).min(document.page_count());
            }
        }
    }

    /// Navigate to first page
    pub fn first_page(&mut self) {
        self.current_page = 0;
    }

    /// Navigate to last page
    pub fn last_page(&mut self) {
        if let Some(document) = &self.document {
            self.current_page = document.page_count().saturating_sub(1);
        }
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.25, 4.0);
    }

    /// Set page margins
    pub fn set_margins(&mut self, margins: boomaga_core::MarginMode) {
        self.margins = margins;
    }

    /// Set pages per sheet
    pub fn set_pages_per_sheet(&mut self, count: boomaga_core::PagesPerSheet) {
        self.pages_per_sheet = count;
    }

    /// Set duplex mode
    pub fn set_duplex(&mut self, mode: boomaga_core::DuplexMode) {
        self.duplex_mode = mode;
    }
}
