//! Main application

use druid::{AppLauncher, Data, Env, Lens};
use std::path::PathBuf;

/// Main application state
#[derive(Clone, Data, Lens)]
pub struct BoomagaApp {
    /// Path to the document to preview
    pub document_path: PathBuf,
    /// Current document
    pub current_document: Option<boomaga_core::Document>,
    /// Current page
    pub current_page: usize,
    /// Zoom level
    pub zoom_level: f64,
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

impl BoomagaApp {
    /// Create a new application
    pub fn new() -> Self {
        Self {
            document_path: PathBuf::new(),
            current_document: None,
            current_page: 0,
            zoom_level: 1.0,
            margins: boomaga_core::MarginMode::Normal,
            pages_per_sheet: boomaga_core::PagesPerSheet::One,
            duplex_mode: boomaga_core::DuplexMode::None,
            print_options: boomaga_core::PrintOptions::default(),
            job_history: Vec::new(),
        }
    }

    /// Load a document
    pub async fn load_document(&mut self, path: PathBuf) -> anyhow::Result<()> {
        self.document_path = path.clone();

        // TODO: Parse document
        // In production, this would:
        // 1. Create Document from path
        // 2. Parse metadata
        // 3. Render preview pages
        // 4. Update current_document

        tracing::info!("Loading document: {:?}", path);

        Ok(())
    }

    /// Navigate to previous page
    pub fn previous_page(&mut self) {
        if let Some(document) = &self.current_document {
            if self.current_page > 0 {
                self.current_page = self.current_page.saturating_sub(1);
            }
        }
    }

    /// Navigate to next page
    pub fn next_page(&mut self) {
        if let Some(document) = &self.current_document {
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
        if let Some(document) = &self.current_document {
            self.current_page = document.page_count().saturating_sub(1);
        }
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f64) {
        // Clamp zoom level
        self.zoom_level = zoom.clamp(0.25, 4.0);
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

    /// Print the document
    pub async fn print_document(&self) -> anyhow::Result<()> {
        // TODO: Print document
        tracing::info!("Printing document: {:?}", self.document_path);
        Ok(())
    }

    /// Cancel current job
    pub async fn cancel_job(&self) -> anyhow::Result<()> {
        // TODO: Cancel job
        tracing::info!("Cancelling job");
        Ok(())
    }

    /// Open print dialog
    pub fn open_print_dialog(&mut self) {
        // TODO: Open print dialog
        tracing::info!("Opening print dialog");
    }

    /// Open settings dialog
    pub fn open_settings(&mut self) {
        // TODO: Open settings dialog
        tracing::info!("Opening settings");
    }
}
