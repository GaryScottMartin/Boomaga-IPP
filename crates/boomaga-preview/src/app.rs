//! Preview application state (the Xilem app model).
//!
//! Widget-free state and transitions. Xilem drives the UI by re-running
//! `app_logic` (see `main.rs`) and delivers renderer events through the worker
//! channel stored here. Matches the `AppData` in `docs/uml/C2-class.puml`.

use boomaga_core::{
    Document, DuplexMode, JobId, JobStatus, PageRange, PageSize, PagesPerSheet, PrintOptions,
};
use boomaga_ipc::MessagePayload;
use boomaga_layout_engine::{BookletPlan, NUpCalculator};
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use crate::ipc_worker::{IpcCommand, IpcEvent, IpcSender};
use crate::pdf_canvas::CanvasImage;
use crate::print_worker::{PrintCommand, PrintEvent, PrintSender, PrinterCapabilities};
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

/// Active preview imposition strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpositionMode {
    NUp,
    Booklet,
}

/// Connection state for backend job notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcState {
    Connecting,
    Connected,
    Disconnected,
}

/// State of downstream printer discovery/submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintState {
    Discovering,
    Ready,
    Submitting,
    Error,
}

/// Keyboard commands accepted by the focused preview canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewShortcut {
    NextPage,
    PreviousPage,
    ZoomIn,
    ZoomOut,
    ResetZoom,
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
    /// N-up or saddle-stitch booklet preview.
    pub imposition_mode: ImpositionMode,
    /// Imposition / print options.
    pub print_options: PrintOptions,
    pub page_range_input: String,
    /// Downstream CUPS destinations and selected destination.
    pub printers: Vec<String>,
    pub selected_printer: usize,
    pub print_state: PrintState,
    pub print_message: Option<String>,
    pub printer_capabilities: Option<(String, PrinterCapabilities)>,
    pub printer_capabilities_pending: bool,
    /// Ids of jobs submitted this session.
    pub job_history: Vec<JobId>,
    /// Latest status received for each backend job.
    pub job_statuses: HashMap<String, JobStatus>,
    /// Current backend notification connection state.
    pub ipc_state: IpcState,
    /// Most recent IPC connection error.
    pub ipc_error: Option<String>,
    print_sender: Option<PrintSender>,
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
            page_range_input: String::new(),
            printers: Vec::new(),
            selected_printer: 0,
            print_state: PrintState::Discovering,
            print_message: None,
            printer_capabilities: None,
            printer_capabilities_pending: false,
            job_history: Vec::new(),
            job_statuses: HashMap::new(),
            ipc_state: IpcState::Disconnected,
            ipc_error: None,
            print_sender: None,
            renderer_sender: None,
            pending_document_path: None,
            render_generation: 0,
            rendering_pages: BTreeSet::new(),
            imposition_revision: 0,
            fill_order: FillOrder::Horizontal,
            imposition_mode: ImpositionMode::NUp,
        }
    }
}

impl AppData {
    /// Apply a keyboard command delivered by the preview canvas.
    pub fn apply_shortcut(&mut self, shortcut: PreviewShortcut) {
        match shortcut {
            PreviewShortcut::NextPage => self.next_page(),
            PreviewShortcut::PreviousPage => self.previous_page(),
            PreviewShortcut::ZoomIn => self.zoom_in(),
            PreviewShortcut::ZoomOut => self.zoom_out(),
            PreviewShortcut::ResetZoom => self.reset_zoom(),
        }
    }

    pub fn install_print_worker(&mut self, sender: PrintSender) {
        self.print_sender = Some(sender);
        self.refresh_printers();
    }

    pub fn refresh_printers(&mut self) {
        self.print_state = PrintState::Discovering;
        self.print_message = None;
        if !self.send_print_command(PrintCommand::Discover) {
            self.print_state = PrintState::Error;
        }
    }

    pub fn handle_print_event(&mut self, event: PrintEvent) {
        match event {
            PrintEvent::Printers(printers) => {
                self.printers = printers;
                self.selected_printer = self
                    .selected_printer
                    .min(self.printers.len().saturating_sub(1));
                self.print_state = PrintState::Ready;
                self.print_message = self
                    .printers
                    .is_empty()
                    .then(|| "No CUPS printers found".to_owned());
                self.refresh_selected_printer_capabilities();
            }
            PrintEvent::Capabilities {
                printer,
                capabilities,
            } => {
                if self.selected_printer_name() == Some(printer.as_str()) {
                    self.printer_capabilities_pending = false;
                    if !capabilities.supports_duplex {
                        self.print_options.duplex = DuplexMode::None;
                    }
                    if !capabilities.supports_collate {
                        self.print_options.collate = false;
                    }
                    self.printer_capabilities = Some((printer, capabilities));
                }
            }
            PrintEvent::CapabilitiesFailed { printer, message } => {
                if self.selected_printer_name() == Some(printer.as_str()) {
                    self.printer_capabilities_pending = false;
                    self.printer_capabilities = None;
                    self.print_message =
                        Some(format!("Could not read {printer} capabilities: {message}"));
                }
            }
            PrintEvent::Submitted(message) => {
                self.print_state = PrintState::Ready;
                self.print_message = Some(if message.is_empty() {
                    "Print job submitted".to_owned()
                } else {
                    message
                });
            }
            PrintEvent::Failed(message) => {
                self.print_state = PrintState::Error;
                self.print_message = Some(message);
            }
        }
    }

    pub fn selected_printer_name(&self) -> Option<&str> {
        self.printers.get(self.selected_printer).map(String::as_str)
    }

    pub fn select_next_printer(&mut self) {
        if !self.printers.is_empty() {
            self.selected_printer = (self.selected_printer + 1) % self.printers.len();
            self.refresh_selected_printer_capabilities();
        }
    }

    fn refresh_selected_printer_capabilities(&mut self) {
        self.printer_capabilities = None;
        self.printer_capabilities_pending = false;
        if let Some(printer) = self.selected_printer_name().map(str::to_owned) {
            self.printer_capabilities_pending =
                self.send_print_command(PrintCommand::DiscoverCapabilities { printer });
        }
    }

    pub fn selected_printer_capabilities(&self) -> Option<PrinterCapabilities> {
        let printer = self.selected_printer_name()?;
        self.printer_capabilities
            .as_ref()
            .and_then(|(name, capabilities)| (name == printer).then_some(*capabilities))
    }

    pub fn decrement_copies(&mut self) {
        self.print_options.copies = self.print_options.copies.saturating_sub(1).max(1);
    }

    pub fn increment_copies(&mut self) {
        self.print_options.copies = self.print_options.copies.saturating_add(1);
    }

    pub fn toggle_collate(&mut self) {
        if self.printer_capabilities_pending {
            return;
        }
        if self
            .selected_printer_capabilities()
            .is_none_or(|caps| caps.supports_collate)
        {
            self.print_options.collate = !self.print_options.collate;
        }
    }

    pub fn set_page_range_input(&mut self, input: String) {
        if self.page_range_input == input {
            return;
        }
        self.page_range_input = input;
        self.current_page = 0;
        self.imposition_revision = self.imposition_revision.wrapping_add(1);
        self.request_current_page();
    }

    pub fn cycle_duplex(&mut self) {
        if self.printer_capabilities_pending {
            return;
        }
        if self
            .selected_printer_capabilities()
            .is_some_and(|caps| !caps.supports_duplex)
        {
            return;
        }
        self.print_options.duplex = match self.print_options.duplex {
            DuplexMode::None => DuplexMode::LongEdge,
            DuplexMode::LongEdge => DuplexMode::ShortEdge,
            DuplexMode::ShortEdge => DuplexMode::None,
        };
    }

    pub fn submit_print_job(&mut self) {
        let Some(printer) = self.selected_printer_name().map(str::to_owned) else {
            self.print_state = PrintState::Error;
            self.print_message = Some("Select a downstream printer first".to_owned());
            return;
        };
        let Some(document) = self.document_path.clone() else {
            self.print_state = PrintState::Error;
            self.print_message = Some("Open a PDF before printing".to_owned());
            return;
        };
        let page_count = self.document.as_ref().map_or(0, Document::page_count);
        self.print_options.page_range = if self.page_range_input.trim().is_empty() {
            None
        } else {
            match self.page_range_input.parse::<PageRange>() {
                Ok(selection) => Some(selection),
                Err(error) => {
                    self.print_state = PrintState::Error;
                    self.print_message = Some(error.to_string());
                    return;
                }
            }
        };
        let selected_pages = if let Some(selection) = &self.print_options.page_range {
            match selection.pages(page_count) {
                Ok(pages) => pages,
                Err(error) => {
                    self.print_state = PrintState::Error;
                    self.print_message = Some(error.to_string());
                    return;
                }
            }
        } else {
            (1..=page_count).collect()
        };
        if let Err(error) = self.print_options.validate() {
            self.print_state = PrintState::Error;
            self.print_message = Some(error.to_string());
            return;
        }
        let booklet_sides = if self.imposition_mode == ImpositionMode::Booklet {
            if self
                .selected_printer_capabilities()
                .is_some_and(|capabilities| !capabilities.supports_duplex)
            {
                self.print_state = PrintState::Error;
                self.print_message =
                    Some("Selected printer does not support booklet duplex".into());
                return;
            }
            match BookletPlan::new(selected_pages.len()) {
                Ok(plan) => Some(
                    plan.sides
                        .into_iter()
                        .map(|side| {
                            side.slots.map(|slot| {
                                slot.map(|index| selected_pages[index].saturating_sub(1))
                            })
                        })
                        .collect(),
                ),
                Err(error) => {
                    self.print_state = PrintState::Error;
                    self.print_message = Some(error.to_string());
                    return;
                }
            }
        } else {
            None
        };

        self.print_state = PrintState::Submitting;
        self.print_message = Some(format!("Submitting to {printer}…"));
        let options = self.print_options.clone();
        if !self.send_print_command(PrintCommand::Submit {
            printer,
            document,
            page_count,
            options,
            booklet_sides,
        }) {
            self.print_state = PrintState::Error;
        }
    }

    fn send_print_command(&mut self, command: PrintCommand) -> bool {
        let sent = self
            .print_sender
            .as_ref()
            .is_some_and(|sender| sender.send(command).is_ok());
        if !sent {
            self.print_message = Some("Print worker is unavailable".to_owned());
        }
        sent
    }

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
        self.current_sheet_slots()
            .into_iter()
            .map(|slot| {
                slot.and_then(|page_index| self.rendered_pages.get(page_index).cloned().flatten())
            })
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

    /// Start receiving backend notifications on the configured Unix socket.
    pub fn install_ipc(&mut self, sender: IpcSender) {
        self.ipc_state = IpcState::Connecting;
        if sender
            .send(IpcCommand::Connect(PathBuf::from(
                boomaga_core::constants::IPC_SOCKET_PATH,
            )))
            .is_err()
        {
            self.ipc_state = IpcState::Disconnected;
            self.ipc_error = Some("IPC worker is unavailable".to_owned());
        }
    }

    /// Apply a backend notification delivered on Xilem's UI thread.
    pub fn handle_ipc_event(&mut self, event: IpcEvent) {
        match event {
            IpcEvent::Message(message) => {
                self.ipc_state = IpcState::Connected;
                self.ipc_error = None;
                if let MessagePayload::PrintJobStatus { job_id, status } = message.payload {
                    let key = job_id.to_string();
                    if !self.job_statuses.contains_key(&key) {
                        self.job_history.push(job_id);
                    }
                    self.job_statuses.insert(key, status);
                }
            }
            IpcEvent::Disconnected(error) => {
                self.ipc_state = IpcState::Disconnected;
                self.ipc_error = Some(error);
            }
        }
    }

    /// Most recently seen backend job and its current status.
    pub fn latest_job_status(&self) -> Option<(&JobId, JobStatus)> {
        let job_id = self.job_history.last()?;
        let status = *self.job_statuses.get(&job_id.to_string())?;
        Some((job_id, status))
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
        for page_index in self.current_sheet_slots().into_iter().flatten() {
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

    fn selected_source_pages(&self) -> Vec<usize> {
        let page_count = self.source_page_count();
        if self.page_range_input.trim().is_empty() {
            return (0..page_count).collect();
        }

        self.page_range_input
            .parse::<PageRange>()
            .and_then(|selection| selection.pages(page_count))
            .map(|pages| pages.into_iter().map(|page| page - 1).collect())
            .unwrap_or_else(|_| (0..page_count).collect())
    }

    pub fn current_sheet_pages(&self) -> Vec<usize> {
        self.current_sheet_slots().into_iter().flatten().collect()
    }

    pub fn current_sheet_slots(&self) -> Vec<Option<usize>> {
        self.sheet_pages()
            .get(self.current_page)
            .cloned()
            .unwrap_or_default()
    }

    fn sheet_pages(&self) -> Vec<Vec<Option<usize>>> {
        let pages = self.selected_source_pages();
        if self.imposition_mode == ImpositionMode::Booklet {
            return BookletPlan::new(pages.len())
                .map(|plan| {
                    plan.sides
                        .into_iter()
                        .map(|side| {
                            side.slots
                                .map(|slot| slot.map(|index| pages[index]))
                                .into_iter()
                                .collect()
                        })
                        .collect()
                })
                .unwrap_or_default();
        }

        NUpCalculator::new(self.print_options.pages_per_sheet as u8)
            .and_then(|calculator| calculator.calculate(&pages, PageSize::A4))
            .map(|layout| {
                layout
                    .pages
                    .into_iter()
                    .map(|page| page.input_pages.into_iter().map(Some).collect())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Number of imposed sheets in the loaded document (0 if none).
    pub fn page_count(&self) -> usize {
        self.sheet_pages().len()
    }

    pub fn set_pages_per_sheet(&mut self, pages_per_sheet: PagesPerSheet) {
        if self.imposition_mode == ImpositionMode::NUp
            && self.print_options.pages_per_sheet == pages_per_sheet
        {
            return;
        }
        self.imposition_mode = ImpositionMode::NUp;
        self.print_options.pages_per_sheet = pages_per_sheet;
        self.current_page = 0;
        self.imposition_revision = self.imposition_revision.wrapping_add(1);
        self.request_current_page();
    }

    pub fn set_booklet_mode(&mut self) {
        if self.imposition_mode == ImpositionMode::Booklet {
            return;
        }
        self.imposition_mode = ImpositionMode::Booklet;
        self.print_options.pages_per_sheet = PagesPerSheet::Two;
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
    use boomaga_ipc::{Message, MessageDestination, MessageSource};

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
    fn preview_shortcuts_drive_navigation_and_zoom() {
        let mut data = AppData {
            document: Some(document_with_pages(3)),
            ..AppData::default()
        };

        data.apply_shortcut(PreviewShortcut::NextPage);
        assert_eq!(data.current_page, 1);
        data.apply_shortcut(PreviewShortcut::PreviousPage);
        assert_eq!(data.current_page, 0);

        data.apply_shortcut(PreviewShortcut::ZoomIn);
        assert!(data.zoom > 1.0);
        data.apply_shortcut(PreviewShortcut::ZoomOut);
        assert!((data.zoom - 1.0).abs() < f64::EPSILON);
        data.apply_shortcut(PreviewShortcut::ZoomIn);
        data.apply_shortcut(PreviewShortcut::ResetZoom);
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

    #[test]
    fn job_notifications_update_existing_status_without_duplicates() {
        let job_id: JobId =
            serde_json::from_str("\"f7f04d62-a28d-4f7c-a55a-cf35dc913918\"").unwrap();
        let mut data = AppData::default();

        for status in [JobStatus::Queued, JobStatus::Processing] {
            data.handle_ipc_event(IpcEvent::Message(Message::new_notification(
                MessageSource::Backend,
                MessageDestination::Preview,
                MessagePayload::PrintJobStatus {
                    job_id: job_id.clone(),
                    status,
                },
            )));
        }

        assert_eq!(data.ipc_state, IpcState::Connected);
        assert_eq!(data.job_history.len(), 1);
        let (latest_id, latest_status) = data.latest_job_status().unwrap();
        assert_eq!(latest_id.to_string(), job_id.to_string());
        assert_eq!(latest_status, JobStatus::Processing);
    }

    #[test]
    fn capability_discovery_marks_controls_pending_and_targets_selected_printer() {
        let mut data = AppData {
            printers: vec!["Office".into(), "Photo".into()],
            ..AppData::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        data.install_print_worker(sender);
        assert!(matches!(
            receiver.try_recv().unwrap(),
            PrintCommand::Discover
        ));

        data.handle_print_event(PrintEvent::Printers(data.printers.clone()));
        assert!(data.printer_capabilities_pending);
        assert!(matches!(
            receiver.try_recv().unwrap(),
            PrintCommand::DiscoverCapabilities { printer } if printer == "Office"
        ));

        data.select_next_printer();
        assert!(data.printer_capabilities_pending);
        assert!(matches!(
            receiver.try_recv().unwrap(),
            PrintCommand::DiscoverCapabilities { printer } if printer == "Photo"
        ));
    }

    #[test]
    fn capability_controls_wait_for_discovery_and_enforce_unsupported_options() {
        let mut data = AppData {
            printers: vec!["Office".into()],
            printer_capabilities_pending: true,
            ..AppData::default()
        };

        data.toggle_collate();
        data.cycle_duplex();
        assert!(!data.print_options.collate);
        assert_eq!(data.print_options.duplex, DuplexMode::None);

        data.print_options.collate = true;
        data.print_options.duplex = DuplexMode::LongEdge;
        data.handle_print_event(PrintEvent::Capabilities {
            printer: "Office".into(),
            capabilities: PrinterCapabilities::default(),
        });
        assert!(!data.printer_capabilities_pending);
        assert!(!data.print_options.collate);
        assert_eq!(data.print_options.duplex, DuplexMode::None);

        data.toggle_collate();
        data.cycle_duplex();
        assert!(!data.print_options.collate);
        assert_eq!(data.print_options.duplex, DuplexMode::None);
    }

    #[test]
    fn stale_capability_results_do_not_change_selected_printer_options() {
        let capabilities = PrinterCapabilities {
            supports_duplex: true,
            supports_collate: true,
        };
        let mut data = AppData {
            printers: vec!["Office".into(), "Photo".into()],
            selected_printer: 1,
            printer_capabilities_pending: true,
            print_options: PrintOptions {
                collate: true,
                duplex: DuplexMode::LongEdge,
                ..PrintOptions::default()
            },
            ..AppData::default()
        };

        data.handle_print_event(PrintEvent::Capabilities {
            printer: "Office".into(),
            capabilities: PrinterCapabilities::default(),
        });
        assert!(data.printer_capabilities_pending);
        assert!(data.print_options.collate);
        assert_eq!(data.print_options.duplex, DuplexMode::LongEdge);

        data.handle_print_event(PrintEvent::Capabilities {
            printer: "Photo".into(),
            capabilities,
        });
        assert_eq!(data.selected_printer_capabilities(), Some(capabilities));
        assert!(!data.printer_capabilities_pending);
    }

    #[test]
    fn booklet_preview_uses_fold_order_and_preserves_blank_slots() {
        let mut data = AppData {
            document: Some(document_with_pages(6)),
            ..AppData::default()
        };

        data.set_booklet_mode();

        assert_eq!(data.imposition_mode, ImpositionMode::Booklet);
        assert_eq!(data.print_options.pages_per_sheet, PagesPerSheet::Two);
        assert_eq!(data.page_count(), 4);
        assert_eq!(data.current_sheet_slots(), vec![None, Some(0)]);
        data.next_page();
        assert_eq!(data.current_sheet_slots(), vec![Some(1), None]);
        data.last_page();
        assert_eq!(data.current_sheet_slots(), vec![Some(3), Some(4)]);
    }

    #[test]
    fn booklet_submission_maps_selected_pages_into_fold_order() {
        let mut data = AppData {
            document_path: Some(PathBuf::from("selected.pdf")),
            document: Some(document_with_pages(6)),
            printers: vec!["Office".into()],
            page_range_input: "2-5".into(),
            ..AppData::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        data.install_print_worker(sender);
        assert!(matches!(
            receiver.try_recv().unwrap(),
            PrintCommand::Discover
        ));
        data.set_booklet_mode();

        data.submit_print_job();

        match receiver.try_recv().unwrap() {
            PrintCommand::Submit {
                page_count,
                booklet_sides,
                ..
            } => {
                assert_eq!(page_count, 6);
                assert_eq!(
                    booklet_sides.unwrap(),
                    vec![[Some(4), Some(1)], [Some(2), Some(3)]]
                );
            }
            command => panic!("unexpected print command: {command:?}"),
        }
        assert_eq!(data.print_state, PrintState::Submitting);
    }

    #[test]
    fn page_range_updates_booklet_preview_to_match_submission() {
        let mut data = AppData {
            document: Some(document_with_pages(6)),
            ..AppData::default()
        };
        data.set_booklet_mode();

        data.set_page_range_input("2-5".into());

        assert_eq!(data.page_count(), 2);
        assert_eq!(data.current_sheet_slots(), vec![Some(4), Some(1)]);
        data.next_page();
        assert_eq!(data.current_sheet_slots(), vec![Some(2), Some(3)]);
    }

    #[test]
    fn invalid_page_range_keeps_full_preview_until_input_is_valid() {
        let mut data = AppData {
            document: Some(document_with_pages(6)),
            ..AppData::default()
        };
        data.set_booklet_mode();

        data.set_page_range_input("2-".into());

        assert_eq!(data.page_count(), 4);
        assert_eq!(data.current_sheet_slots(), vec![None, Some(0)]);
    }

    #[test]
    fn choosing_n_up_exits_booklet_mode() {
        let mut data = AppData {
            document: Some(document_with_pages(8)),
            ..AppData::default()
        };
        data.set_booklet_mode();

        data.set_pages_per_sheet(PagesPerSheet::Four);

        assert_eq!(data.imposition_mode, ImpositionMode::NUp);
        assert_eq!(data.page_count(), 2);
        assert_eq!(data.current_sheet_pages(), vec![0, 1, 2, 3]);
    }
}
