//! Document viewer component

use druid::{
    Color, Env, ExtEventSink, PaintCtx, Point, Rect, RenderContext, Size, Widget,
};
use cairo::{Context, Format, ImageSurface};

/// Document viewer widget
pub struct DocumentViewer {
    /// Current document
    document: Option<boomaga_core::Document>,
    /// Current page
    current_page: usize,
    /// Zoom level
    zoom: f64,
}

impl DocumentViewer {
    /// Create a new document viewer
    pub fn new() -> Self {
        Self {
            document: None,
            current_page: 0,
            zoom: 1.0,
        }
    }

    /// Set document
    pub fn set_document(&mut self, document: boomaga_core::Document) {
        self.document = Some(document);
        self.current_page = 0;
    }

    /// Set current page
    pub fn set_page(&mut self, page: usize) {
        if let Some(document) = &self.document {
            self.current_page = page.min(document.page_count());
        }
    }

    /// Set zoom
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom;
    }
}

impl Default for DocumentViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<BoomagaApp> for DocumentViewer {
    fn event(&mut self, _ctx: &mut druid::EventCtx, _event: &druid::Event, _data: &mut BoomagaApp, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &BoomagaApp, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &BoomagaApp, _data: &BoomagaApp, _env: &Env) {}

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &BoomagaApp, env: &Env) -> druid::Size {
        // Use requested constraints
        ctx.constrain(bc)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &BoomagaApp, env: &Env) {
        // Clear background
        ctx.fill(ctx.size(), &Color::WHITE);

        if let Some(document) = &data.current_document {
            // Draw page
            if let Some(page) = document.pages.get(data.current_page) {
                self.draw_page(ctx, page, ctx.size());
            } else {
                // Draw placeholder
                self.draw_placeholder(ctx, ctx.size());
            }
        } else {
            // Draw empty state
            self.draw_empty_state(ctx, ctx.size());
        }
    }

    /// Draw a page
    fn draw_page(&self, ctx: &mut PaintCtx, page: &boomaga_core::Page, size: Size) {
        let surface = ImageSurface::create(Format::ARgb32, (page.width * self.zoom) as i32, (page.height * self.zoom) as i32)
            .expect("Failed to create image surface");

        let cairo_ctx = Context::new(&surface)
            .expect("Failed to create cairo context");

        // In production, this would render the actual page content
        // For now, draw a placeholder
        cairo_ctx.set_source_rgb(0.9, 0.9, 0.9);
        cairo_ctx.paint();

        // Draw page border
        let rect = Rect::ZERO.with_origin(Point::ORIGIN)
            .with_size(Size::new(page.width * self.zoom, page.height * self.zoom));
        cairo_ctx.set_source_rgb(0.0, 0.0, 0.0);
        cairo_ctx.set_line_width(2.0);
        cairo_ctx.rectangle(
            rect.x0,
            rect.y0,
            rect.width(),
            rect.height(),
        );
        cairo_ctx.stroke();

        // Draw page number
        cairo_ctx.set_source_rgb(0.0, 0.0, 0.0);
        cairo_ctx.set_font_size(14.0);
        cairo_ctx.move_to(10.0, 20.0);
        cairo_ctx.show_text(&format!("Page {}", page.number));

        ctx.submit_image(
            ImageSurface::from_foreign(surface.into_raw() as *mut _),
            Rect::ZERO,
        );
    }

    /// Draw placeholder
    fn draw_placeholder(&self, ctx: &mut PaintCtx, size: Size) {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let radius = 30.0;

        ctx.with_save(|ctx| {
            ctx.translate(center);
            ctx.rotate(std::time::Instant::now().as_secs_f64() / 2.0);

            // Draw rotating circle
            for i in 0..12 {
                let angle = (i as f64) * std::f64::consts::PI / 6.0;
                let x = radius * angle.cos();
                let y = radius * angle.sin();

                ctx.fill(
                    Rect::new(x - 3.0, y - 3.0, 6.0, 6.0),
                    &Color::GRAY50,
                );
            }
        });

        // Draw text
        ctx.draw_text(
            &druid::TextLayout::new("Loading document...")
                .font("Sans")
                .content_size()
                .unwrap_or_default(),
            Point::new(0.0, 0.0),
            &Color::BLACK,
        );
    }

    /// Draw empty state
    fn draw_empty_state(&self, ctx: &mut PaintCtx, size: Size) {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let text_color = Color::GRAY500;

        // Draw icon
        ctx.with_save(|ctx| {
            ctx.translate(center);
            ctx.translate(-text_color.color().unwrap().r as f64 / 255.0,
                         -text_color.color().unwrap().g as f64 / 255.0,
                         -text_color.color().unwrap().b as f64 / 255.0);

            ctx.fill(
                Rect::new(-30.0, -30.0, 60.0, 60.0),
                &text_color,
            );

            ctx.fill(
                Rect::new(-15.0, -15.0, 30.0, 30.0),
                &text_color,
            );
        });

        // Draw text
        ctx.draw_text(
            &druid::TextLayout::new("No document loaded")
                .font("Sans")
                .content_size()
                .unwrap_or_default(),
            Point::new(0.0, 50.0),
            &text_color,
        );
    }
}
