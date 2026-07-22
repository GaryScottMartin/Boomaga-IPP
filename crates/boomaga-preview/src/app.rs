//! Preview application state (the Xilem app model).
//!
//! Widget-free state and transitions. Xilem drives the UI by re-running
//! `app_logic` (see `main.rs`) and delivers renderer events through the worker
//! channel stored here. Matches the `AppData` in `docs/uml/C2-class.puml`.

use boomaga_core::{Document, JobId, PageSize, PagesPerSheet, PrintOptions};
use boomaga_layout_engine::NUpCalculator;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::pdf_canvas::CanvasImage;
use crate::render_worker::{RendererCommand, RendererEvent, RendererSender};

/// Current document-loading state shown by the preview UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Idle,
    Loading,
    Ready,
    Error,
}

/// Page fill order within an imposed sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillOrder {
    Horizontal,
    Vertical,
}

/// Preview application state.
pub struct AppData {
    /// Path of the document being previewed, if any.
    pub document_path: Option<PathBuf>,
    /// The loaded document, if any.
    pub document: Option<Document>,
    /// Zero-based index of the page currently shown.
    pub current_page: usize,
    /// Rasterized pages, ready for the Masonry canvas.
    pub rendered_pages: Vec<Option<CanvasImage>>,
    /// Current document loading state.
    pub load_state: LoadState,
    /// Most recent file-loading or page-rendering error.
    pub error_message: Option<String>,
    /// Whether the native file chooser is currently open.
    pub choosing_file: bool,
    /// Zoom factor (1.0 == 100%).
    pub zoom: f64,
    renderer_sender: Option<RendererSender>,
    pending_document_path: Option<PathBuf>,
    render_generation: u64,
    rendering_pages: BTreeSet<usize>,
    imposition_revision: u64,
    /// Page fill order for multi-page imposed sheets.
    pub fill_order: FillOrder,
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
            load_state: LoadState::Idle,
            error_message: None,
            choosing_file: false,
            print_options: PrintOptions::default(),
            job_history: Vec::new(),
            renderer_sender: None,
            pending_document_path: None,
            render_generation: 0,
            rendering_pages: BTreeSet::new(),
            imposition_revision: 0,
            fill_order: FillOrder::Horizontal,
        }
    }
}

impl AppData {
    /// Create initial state which asynchronously loads a command-line PDF.
    pub fn with_document_path(path: PathBuf) -> Self {
        Self {
            pending_document_path: Some(path),
            ..Self::default()
        }
    }

    /// Rasterized image for the page currently selected, if available.
    pub fn current_canvas_image(&self) -> Option<&CanvasImage> {
        let source_page = self.current_sheet_pages().into_iter().next()?;
        self.rendered_pages
            .get(source_page)
            .and_then(Option::as_ref)
    }

    /// Ordered rendered-image slots for the source pages on the current sheet.
    pub fn current_canvas_images(&self) -> Vec<Option<CanvasImage>> {
        self.current_sheet_pages()
            .into_iter()
            .map(|page_index| self.rendered_pages.get(page_index).cloned().flatten())
            .collect()
    }

    /// Number of pages which have been rendered into the on-demand cache.
    pub fn rendered_page_count(&self) -> usize {
        self.rendered_pages
            .iter()
            .filter(|image| image.is_some())
            .count()
    }

    /// Connect the Xilem worker command channel and start any pending CLI load.
    pub fn install_renderer(&mut self, sender: RendererSender) {
        self.renderer_sender = Some(sender);
        if let Some(path) = self.pending_document_path.take() {
            self.load_document(path);
        }
    }

    /// Open the native PDF chooser without blocking the UI thread.
    pub fn choose_document(&mut self) {
        if self.choosing_file {
            return;
        }
        self.error_message = None;
        self.choosing_file = true;
        if !self.send_command(RendererCommand::OpenFileDialog) {
            self.choosing_file = false;
        }
    }

    /// Reset document state and ask the background renderer to load `path`.
    pub fn load_document(&mut self, path: PathBuf) {
        self.render_generation = self.render_generation.wrapping_add(1);
        self.document_path = Some(path.clone());
        self.document = None;
        self.current_page = 0;
        self.rendered_pages.clear();
        self.rendering_pages.clear();
        self.error_message = None;
        self.load_state = LoadState::Loading;

        self.send_command(RendererCommand::Load {
            generation: self.render_generation,
            path,
        });
    }

    /// Apply a renderer result delivered by Xilem's `MessageProxy`.
    pub fn handle_renderer_event(&mut self, event: RendererEvent) {
        match event {
            RendererEvent::FileSelected(path) => {
                self.choosing_file = false;
                self.load_document(path);
            }
            RendererEvent::FileDialogCancelled => self.choosing_file = false,
            RendererEvent::DocumentLoaded {
                generation,
                path,
                document,
            } => {
                if generation != self.render_generation {
                    return;
                }
                self.document_path = Some(path);
                self.rendered_pages = vec![None; document.page_count()];
                self.document = Some(document);
                self.load_state = LoadState::Ready;
                self.request_current_page();
            }
            RendererEvent::PageRendered {
                generation,
                page_index,
                image,
            } => {
                if generation != self.render_generation {
                    return;
                }
                self.rendering_pages.remove(&page_index);
                self.error_message = None;
                if let Some(slot) = self.rendered_pages.get_mut(page_index) {
                    *slot = Some(image);
                }
            }
            RendererEvent::Failed {
                generation,
                page_index,
                message,
            } => {
                if generation.is_some_and(|value| value != self.render_generation) {
                    return;
                }
                if let Some(page_index) = page_index {
                    self.rendering_pages.remove(&page_index);
                } else {
                    self.load_state = LoadState::Error;
                }
                self.choosing_file = false;
                self.error_message = Some(message);
            }
        }
    }

    fn send_command(&mut self, command: RendererCommand) -> bool {
        let sent = self
            .renderer_sender
            .as_ref()
            .is_some_and(|sender| sender.send(command).is_ok());
        if !sent {
            self.load_state = LoadState::Error;
            self.error_message = Some("PDF renderer is unavailable".to_owned());
        }
        sent
    }

    fn request_current_page(&mut self) {
        for page_index in self.current_sheet_pages() {
            if self.rendered_pages.get(page_index).is_none()
                || self.rendered_pages[page_index].is_some()
                || !self.rendering_pages.insert(page_index)
            {
                continue;
            }
            if !self.send_command(RendererCommand::RenderPage {
                generation: self.render_generation,
                page_index,
            }) {
                self.rendering_pages.remove(&page_index);
            }
        }
    }

    fn source_page_count(&self) -> usize {
        self.document.as_ref().map_or(0, Document::page_count)
    }

    pub fn current_sheet_pages(&self) -> Vec<usize> {
        self.sheet_pages()
            .get(self.current_page)
            .cloned()
            .unwrap_or_default()
    }

    fn sheet_pages(&self) -> Vec<Vec<usize>> {
        let pages: Vec<_> = (0..self.source_page_count()).collect();
        NUpCalculator::new(self.print_options.pages_per_sheet as u8)
            .and_then(|calculator| calculator.calculate(&pages, PageSize::A4))
            .map(|layout| {
                layout
                    .pages
                    .into_iter()
                    .map(|page| page.input_pages)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Number of imposed sheets in the loaded document (0 if none).
    pub fn page_count(&self) -> usize {
        self.sheet_pages().len()
    }

    pub fn set_pages_per_sheet(&mut self, pages_per_sheet: PagesPerSheet) {
        if self.print_options.pages_per_sheet == pages_per_sheet {
            return;
        }
        self.print_options.pages_per_sheet = pages_per_sheet;
        self.current_page = 0;
        self.imposition_revision = self.imposition_revision.wrapping_add(1);
        self.request_current_page();
    }

    pub fn set_fill_order(&mut self, fill_order: FillOrder) {
        if self.fill_order != fill_order {
            self.fill_order = fill_order;
            self.imposition_revision = self.imposition_revision.wrapping_add(1);
        }
    }

    /// Advance to the next page, clamped to the last page.
    pub fn next_page(&mut self) {
        let last = self.page_count().saturating_sub(1);
        if self.current_page < last {
            self.current_page += 1;
        }
        self.request_current_page();
    }

    /// Go to the previous page, clamped to the first page.
    pub fn previous_page(&mut self) {
        self.current_page = self.current_page.saturating_sub(1);
        self.request_current_page();
    }

    /// Jump to the first page.
    pub fn first_page(&mut self) {
        self.current_page = 0;
        self.request_current_page();
    }

    /// Jump to the last page.
    pub fn last_page(&mut self) {
        self.current_page = self.page_count().saturating_sub(1);
        self.request_current_page();
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

    #[test]
    fn command_line_path_is_loaded_after_worker_connects() {
        let path = PathBuf::from("large.pdf");
        let mut data = AppData::with_document_path(path.clone());
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

        data.install_renderer(sender);

        match receiver.try_recv().unwrap() {
            RendererCommand::Load {
                generation,
                path: requested_path,
            } => {
                assert_eq!(generation, 1);
                assert_eq!(requested_path, path);
            }
            command => panic!("unexpected renderer command: {command:?}"),
        }
        assert_eq!(data.load_state, LoadState::Loading);
    }

    #[test]
    fn loaded_document_requests_only_the_current_page() {
        let path = PathBuf::from("three-pages.pdf");
        let mut data = AppData::default();
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        data.install_renderer(sender);
        data.load_document(path.clone());
        let _load_command = receiver.try_recv().unwrap();

        data.handle_renderer_event(RendererEvent::DocumentLoaded {
            generation: 1,
            path,
            document: document_with_pages(3),
        });

        assert_eq!(data.load_state, LoadState::Ready);
        assert_eq!(data.rendered_pages.len(), 3);
        assert_eq!(data.rendered_page_count(), 0);
        match receiver.try_recv().unwrap() {
            RendererCommand::RenderPage {
                generation,
                page_index,
            } => {
                assert_eq!(generation, 1);
                assert_eq!(page_index, 0);
            }
            command => panic!("unexpected renderer command: {command:?}"),
        }
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn two_up_navigation_uses_sheet_count() {
        let mut data = AppData {
            document: Some(document_with_pages(5)),
            ..AppData::default()
        };
        data.set_pages_per_sheet(PagesPerSheet::Two);
        assert_eq!(data.page_count(), 3);
        assert_eq!(data.current_sheet_pages(), vec![0, 1]);
        data.last_page();
        assert_eq!(data.current_sheet_pages(), vec![4]);
    }

    #[test]
    fn changing_n_up_invalidates_imposition_without_discarding_rasters() {
        let image = CanvasImage::from_cairo_bgra(vec![0; 4], 1, 1).unwrap();
        let mut data = AppData {
            document: Some(document_with_pages(2)),
            current_page: 1,
            rendered_pages: vec![Some(image), None],
            ..AppData::default()
        };
        let revision = data.imposition_revision;
        data.set_pages_per_sheet(PagesPerSheet::Two);
        assert_eq!(data.current_page, 0);
        assert_eq!(data.imposition_revision, revision + 1);
        assert_eq!(data.rendered_page_count(), 1);
    }

    #[test]
    fn two_up_requests_every_page_on_current_sheet() {
        let mut data = AppData {
            document: Some(document_with_pages(3)),
            rendered_pages: vec![None; 3],
            ..AppData::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        data.install_renderer(sender);
        data.set_pages_per_sheet(PagesPerSheet::Two);
        for expected_page in [0, 1] {
            match receiver.try_recv().unwrap() {
                RendererCommand::RenderPage { page_index, .. } => {
                    assert_eq!(page_index, expected_page);
                }
                command => panic!("unexpected renderer command: {command:?}"),
            }
        }
        assert!(receiver.try_recv().is_err());
    }
}
