//! Document view widget
//!
//! Displays a single rendered page of a document with zoom support.

use druid::{
    widget::{Label, Flex, Image},
    Data, Env, LifeCycle, LifeCycleCtx, PaintCtx, Point,
    UpdateCtx, Widget, WidgetPod,
};
use std::sync::{Arc, Mutex};
use cairo::{ImageSurface, Context as CairoContext, Format};

use boomaga_core::{Document, Page};

/// Document view widget state
#[derive(Clone, Data)]
pub struct DocumentViewState {
    /// Document being viewed
    pub document: Option<Document>,
    /// Current page number (0-indexed)
    pub current_page: usize,
    /// Zoom level
    pub zoom_level: f64,
    /// Page rendered as image (cached for performance)
    pub rendered_page: Option<Arc<Mutex<Option<ImageSurface>>>>,
    /// Error message (if any)
    pub error: Option<String>,
}

impl Default for DocumentViewState {
    fn default() -> Self {
        Self {
            document: None,
            current_page: 0,
            zoom_level: 1.0,
            rendered_page: None,
            error: None,
        }
    }
}

impl DocumentViewState {
    /// Clear cached rendered page
    pub fn clear_cache(&mut self) {
        self.rendered_page = None;
    }
}

/// Document view widget
pub struct DocumentView {
    widget: WidgetPod<DocumentViewState, druid::Widget<DocumentViewState>>,
}

impl DocumentView {
    /// Create a new document view widget
    pub fn new() -> Self {
        let widget = druid::Widget::default();
        let widget_pod = widget.into_pod();
        Self { widget }
    }

    /// Set a document to view
    pub fn set_document(&mut self, document: Document) {
        let state = DocumentViewState {
            document: Some(document),
            ..Default::default()
        };
        self.widget.as_mut().set_data(state);
    }

    /// Set current page
    pub fn set_current_page(&mut self, page_number: usize) {
        if let Some(state) = self.widget.as_mut().data_mut() {
            if let Some(doc) = state.document.as_ref() {
                if page_number < doc.page_count() {
                    state.current_page = page_number;
                    state.rendered_page = None; // Clear cache
                }
            }
        }
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: f64) {
        if let Some(state) = self.widget.as_mut().data_mut() {
            state.zoom_level = zoom;
            state.rendered_page = None; // Clear cache
        }
    }
}

impl Default for DocumentView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<DocumentViewState> for DocumentView {
    fn event(&mut self, ctx: &mut UpdateCtx, event: &druid::MouseEvent, data: &mut DocumentViewState, env: &Env) {
        self.widget.as_mut().event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &DocumentViewState, env: &Env) {
        self.widget.as_mut().lifecycle(ctx, event, data, env);
    }

    fn handle_event(&mut self, ctx: &mut UpdateCtx, event: &druid::Event, data: &DocumentViewState, env: &Env) {
        self.widget.as_mut().handle_event(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, data: &DocumentViewState, env: &Env) {
        self.widget.as_mut().update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraint, data: &DocumentViewState, env: &Env) -> druid::Size {
        self.widget.as_mut().layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DocumentViewState, env: &Env) {
        // Draw background
        ctx.fill(ctx.size(), &env.get(druid::theme::WINDOW_BACKGROUND_COLOR));

        // Get page size
        let page_size = if let Some(document) = data.document.as_ref() {
            if data.current_page < document.page_count() {
                let page = &document.pages()[data.current_page];
                (page.width, page.height)
            } else {
                (595.0, 842.0) // A4 default
            }
        } else {
            (595.0, 842.0) // A4 default
        };

        // Calculate display size based on zoom
        let display_width = page_size.0 * data.zoom_level;
        let display_height = page_size.1 * data.zoom_level;

        // Center the page
        let canvas_width = ctx.size().width;
        let canvas_height = ctx.size().height;
        let x = (canvas_width - display_width) / 2.0;
        let y = (canvas_height - display_height) / 2.0;

        // Render the page
        match self.render_page(ctx, data, x, y, page_size.0, page_size.1) {
            Ok(image_surface) => {
                let size = Size::new(display_width, display_height);
                let image = Image::new(image_surface)
                    .width(display_width)
                    .height(display_height);

                let container = Container::new(image)
                    .padding(0.0)
                    .fix_size(size);

                container.paint(ctx, data, env);
            }
            Err(e) => {
                ctx.render_text(&format!("Error rendering page: {}", e), Point::ZERO, &env);
            }
        }
    }
}

impl DocumentView {
    /// Render a single page to a Cairo image surface
    fn render_page(
        &self,
        ctx: &mut PaintCtx,
        data: &DocumentViewState,
        x: f64,
        y: f64,
        width_points: f64,
        height_points: f64,
    ) -> Result<ImageSurface, String> {
        // Check if we have a cached render
        if let Some(cached) = &data.rendered_page {
            let cached_lock = cached.lock();
            if cached_lock.is_some() {
                let surface = cached_lock.as_ref().unwrap().clone();
                return Ok(surface);
            }
        }

        // Note: In production, this would use poppler to render the page
        // For now, we'll create a simple placeholder render
        let surface = ImageSurface::create(Format::ARgb32, width_points as i32, height_points as i32)
            .map_err(|e| format!("Failed to create surface: {}", e))?;

        let cairo_ctx = CairoContext::new(&surface);

        // Draw background
        cairo_ctx.set_source_rgb(1.0, 1.0, 1.0);
        cairo_ctx.paint();

        // Draw document info
        cairo_ctx.set_source_rgb(0.0, 0.0, 0.0);
        cairo_ctx.set_font_size(14.0);

        if let Some(document) = data.document.as_ref() {
            let page_text = format!("Document: {}", document.file_path.display());
            cairo_ctx.move_to(50.0, 50.0);
            cairo_ctx.show_text(&page_text);

            let info_text = format!("Pages: {}", document.page_count());
            cairo_ctx.move_to(50.0, 75.0);
            cairo_ctx.show_text(&info_text);

            if let Some(author) = &document.author {
                let author_text = format!("Author: {}", author);
                cairo_ctx.move_to(50.0, 100.0);
                cairo_ctx.show_text(&author_text);
            }
        }

        // Draw page number
        let page_text = format!("Page {}/{}", data.current_page + 1, data.document.as_ref().map(|d| d.page_count()).unwrap_or(1));
        cairo_ctx.move_to(ctx.size().width - 150.0, ctx.size().height - 50.0);
        cairo_ctx.show_text(&page_text);

        // Draw page border
        let page_rect = Rect::new(x, y, display_width, display_height);
        cairo_ctx.set_source_rgb(0.0, 0.0, 0.0);
        cairo_ctx.set_line_width(2.0);
        cairo_ctx.stroke_rectangle(page_rect.x0, page_rect.y0, page_rect.width(), page_rect.height());

        // Cache the rendered surface
        if let Some(cached) = &mut data.rendered_page {
            let cached_lock = cached.lock();
            if cached_lock.is_none() {
                *cached_lock = Some(Arc::new(Mutex::new(surface)));
            }
        }

        Ok(surface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_view_state() {
        let state = DocumentViewState::default();
        assert_eq!(state.current_page, 0);
        assert_eq!(state.zoom_level, 1.0);
    }

    #[test]
    fn test_page_navigation() {
        let mut state = DocumentViewState::default();
        state.current_page = 0;
        state.set_current_page(1);
        assert_eq!(state.current_page, 1);
    }

    #[test]
    fn test_zoom_level() {
        let mut state = DocumentViewState::default();
        state.set_zoom(2.5);
        assert_eq!(state.zoom_level, 2.5);
    }
}
