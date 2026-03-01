//! Document viewer widget
//!
//! Provides a Druid widget for displaying rendered document pages with zoom
//! and navigation controls.

use druid::{
    widget::{Container, Flex, Label, Scroll, Slider},
    Data, Env, LifeCycle, LifeCycleCtx, PaintCtx, Point, Size,
    UpdateCtx, Widget, WidgetPod,
};
use std::path::PathBuf;
use tracing::{info, warn};

use boomaga_core::{Document, Page};
use super::document_renderer::{DocumentRenderer, RenderError, FileType};
use super::document_view::DocumentView;

/// Viewer widget state
#[derive(Clone, Data)]
pub struct ViewerState {
    /// Path to the current document
    pub document_path: Option<PathBuf>,
    /// Loaded document
    pub document: Option<Document>,
    /// Current page number (0-indexed)
    pub current_page: usize,
    /// Zoom level
    pub zoom_level: f64,
    /// Error message (if any)
    pub error: Option<String>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            document_path: None,
            document: None,
            current_page: 0,
            zoom_level: 1.0,
            error: None,
        }
    }
}

/// Document viewer widget
pub struct Viewer {
    renderer: DocumentRenderer,
    doc_view: WidgetPod<DocumentView, druid::Widget<DocumentView>>,
    zoom_slider: f64,
}

impl Viewer {
    /// Create a new viewer widget
    pub fn new() -> Self {
        let renderer = DocumentRenderer::new(uuid::Uuid::new_v4().to_string());
        let doc_view = DocumentView::new().into_pod();
        let zoom_slider = 1.0;

        Self {
            renderer,
            doc_view,
            zoom_slider,
        }
    }

    /// Set a document to view
    pub fn set_document(&mut self, path: PathBuf) {
        tracing::info!("Setting document to view: {:?}", path);

        // Load document using renderer
        match self.renderer.load(&path) {
            Ok(document) => {
                self.zoom_slider = 1.0;
                self.doc_view.as_mut().set_document(document);
            }
            Err(e) => {
                tracing::error!("Failed to load document: {}", e);
                // We'll handle this in the widget's update method
            }
        }
    }

    /// Navigate to a specific page
    pub fn set_page(&mut self, page_number: usize) {
        if let Some(document) = self.doc_view.as_ref().data().document.as_ref() {
            if page_number < document.page_count() {
                self.doc_view.as_mut().set_current_page(page_number);
            }
        }
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom_slider = zoom.clamp(0.25, 4.0);
        self.doc_view.as_mut().set_zoom(zoom);
    }

    /// Go to next page
    pub fn next_page(&mut self) {
        if let Some(document) = self.doc_view.as_ref().data().document.as_ref() {
            if document.page_count() > 0 {
                let current = self.doc_view.as_ref().data().current_page;
                if current < document.page_count() - 1 {
                    self.set_page(current + 1);
                }
            }
        }
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        if let Some(document) = self.doc_view.as_ref().data().document.as_ref() {
            if document.page_count() > 0 {
                let current = self.doc_view.as_ref().data().current_page;
                if current > 0 {
                    self.set_page(current - 1);
                }
            }
        }
    }
}

impl Default for Viewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<ViewerState> for Viewer {
    fn event(&mut self, ctx: &mut UpdateCtx, event: &druid::MouseEvent, data: &mut ViewerState, env: &Env) {
        // Handle mouse wheel for zoom
        if let druid::Event::MouseWheel(raw) = event {
            // We'll handle this in paint or handle_event
        }

        self.doc_view.as_mut().event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &ViewerState, env: &Env) {
        self.doc_view.as_mut().lifecycle(ctx, event, data, env);
    }

    fn handle_event(&mut self, ctx: &mut UpdateCtx, event: &druid::Event, data: &ViewerState, env: &Env) {
        // Handle mouse wheel for page navigation and zoom
        match event {
            druid::Event::MouseWheel(wheel) => {
                let zoom_delta = -wheel.delta.y / 100.0;
                let new_zoom = data.zoom_level * (1.0 + zoom_delta);
                data.zoom_level = new_zoom.clamp(0.25, 4.0);
                ctx.request_paint();
            }
            _ => {}
        }

        self.doc_view.as_mut().handle_event(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, data: &ViewerState, env: &Env) {
        // Update document view when data changes
        self.doc_view.as_mut().update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &druid::BoxConstraint, data: &ViewerState, env: &Env) -> druid::Size {
        let doc_view_layout = self.doc_view.as_mut().layout(ctx, bc, data, env);

        // Return size that fits within constraints
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ViewerState, env: &Env) {
        // Draw background
        ctx.fill(ctx.size(), &env.get(druid::theme::WINDOW_BACKGROUND_COLOR));

        // Draw document view
        let doc_view_layout = self.doc_view.as_mut().layout(ctx, &druid::BoxConstraint::Unbounded(), data, env);
        let doc_view_rect = Rect::from_origin_size(Point::ZERO, doc_view_layout);

        self.doc_view.as_mut().paint(ctx, data, env);

        // Draw zoom controls at bottom
        let zoom_label = format!("Zoom: {:.0}%", data.zoom_level * 100.0);
        ctx.render_text(&zoom_label, Point::new(20.0, doc_view_layout.height() + 20.0), &env);
    }
}

/// Create a complete viewer layout
pub fn create_viewer_layout() -> impl Widget<ViewerState> {
    let viewer = Viewer::new();
    let state = ViewerState::default();

    // Create scrollable container for document view
    let scroll = Scroll::new(viewer.doc_view.clone());

    // Create page info label
    let page_info = Label::dynamic(|state: &ViewerState| {
        if let Some(document) = state.document.as_ref() {
            format!("{}/{} pages", state.current_page + 1, document.page_count())
        } else {
            "No document loaded".to_string()
        }
    })
    .with_text_size(12.0);

    // Create a flexible layout
    let layout = Flex::column()
        .with_child(scroll)
        .with_child(Flex::row()
            .with_spacing(10.0)
            .with_child(page_info)
            .with_flex_spacer()
            .with_child(Label::dynamic("Navigate with mouse wheel").with_text_size(12.0))
        );

    layout
}

/// Create a preview window layout
pub fn create_preview_layout() -> impl Widget<ViewerState> {
    let viewer = Viewer::new();
    let state = ViewerState::default();

    // Create toolbar
    let toolbar = crate::toolbar::Toolbar::new();

    // Create menu bar
    let menu_bar = crate::menu_bar::MenuBar::new();

    // Create scrollable container for document view
    let scroll = Scroll::new(viewer.doc_view.clone());

    // Create main layout
    let main_layout = Flex::column()
        .with_child(menu_bar)
        .with_child(toolbar)
        .with_spacing(0.0)
        .with_child(scroll);

    main_layout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_state() {
        let state = ViewerState::default();
        assert_eq!(state.current_page, 0);
        assert_eq!(state.zoom_level, 1.0);
        assert!(state.error.is_none());
    }
}
