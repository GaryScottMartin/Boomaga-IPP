//! Preview application state (the Xilem app model).
//!
//! Plain data + pure state transitions — no GUI-framework traits. Xilem drives
//! the UI by re-running `app_logic` (see `main.rs`) against this value and
//! diffing the resulting view tree, so `AppData` deliberately knows nothing
//! about the framework. Matches the `AppData` in `docs/uml/C2-class.puml`.

use boomaga_core::{Document, JobId, PrintOptions};
use std::path::PathBuf;

use crate::pdf_canvas::CanvasImage;

/// Preview application state.
pub struct AppData {
    /// Path of the document being previewed, if any.
    pub document_path: Option<PathBuf>,
    /// The loaded document, if any.
    pub document: Option<Document>,
    /// Zero-based index of the page currently shown.
    pub current_page: usize,
    /// Rasterized pages, ready for the Masonry canvas.
    pub rendered_pages: Vec<CanvasImage>,
    /// Zoom factor (1.0 == 100%).
    pub zoom: f64,
    /// Imposition / print options.
    pub print_options: PrintOptions,
    /// Ids of jobs submitted this session.
    pub job_history: Vec<JobId>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            document_path: None,
            document: None,
            current_page: 0,
            rendered_pages: Vec::new(),
            zoom: 1.0,
            print_options: PrintOptions::default(),
            job_history: Vec::new(),
        }
    }
}

impl AppData {
    /// Rasterized image for the page currently selected, if available.
    pub fn current_canvas_image(&self) -> Option<&CanvasImage> {
        self.rendered_pages.get(self.current_page)
    }

    /// Number of pages in the loaded document (0 if none).
    pub fn page_count(&self) -> usize {
        self.document.as_ref().map_or(0, Document::page_count)
    }

    /// Advance to the next page, clamped to the last page.
    pub fn next_page(&mut self) {
        let last = self.page_count().saturating_sub(1);
        if self.current_page < last {
            self.current_page += 1;
        }
    }

    /// Go to the previous page, clamped to the first page.
    pub fn previous_page(&mut self) {
        self.current_page = self.current_page.saturating_sub(1);
    }

    /// Jump to the first page.
    pub fn first_page(&mut self) {
        self.current_page = 0;
    }

    /// Jump to the last page.
    pub fn last_page(&mut self) {
        self.current_page = self.page_count().saturating_sub(1);
    }

    /// Set the zoom factor, clamped to a sane range.
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.25, 4.0);
    }

    /// Zoom in one step (20%).
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * 1.2);
    }

    /// Zoom out one step (20%).
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / 1.2);
    }

    /// Reset zoom to 100%.
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boomaga_core::{FileType, Orientation, Page};

    fn document_with_pages(page_count: usize) -> Document {
        let mut document = Document::new(
            "test-document".to_string(),
            PathBuf::from("test.pdf"),
            FileType::Pdf,
        );

        for number in 0..page_count {
            document.add_page(Page::new(number, 595.0, 842.0, Orientation::Portrait));
        }

        document
    }

    #[test]
    fn navigation_stays_within_document_bounds() {
        let mut data = AppData {
            document: Some(document_with_pages(3)),
            ..AppData::default()
        };

        data.previous_page();
        assert_eq!(data.current_page, 0);

        data.next_page();
        data.next_page();
        data.next_page();
        assert_eq!(data.current_page, 2);

        data.first_page();
        assert_eq!(data.current_page, 0);

        data.last_page();
        assert_eq!(data.current_page, 2);
    }

    #[test]
    fn navigation_without_a_document_stays_on_first_page() {
        let mut data = AppData::default();

        data.next_page();
        data.last_page();

        assert_eq!(data.current_page, 0);
    }

    #[test]
    fn zoom_is_clamped_and_can_be_reset() {
        let mut data = AppData::default();

        data.set_zoom(0.1);
        assert_eq!(data.zoom, 0.25);

        data.set_zoom(10.0);
        assert_eq!(data.zoom, 4.0);

        data.reset_zoom();
        assert_eq!(data.zoom, 1.0);
    }
}
