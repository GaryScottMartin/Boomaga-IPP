//! Single page container for Xilem

use druid::kurbo::Vec2;
use xilem::{Widget, WidgetId, PaintCtx, Env};
use druid::ImageData;

/// Page container widget
pub struct PageContainer {
    page_image: Option<ImageData>,
    transform: druid::kurbo::Affine,
}

impl PageContainer {
    /// Create a new page container
    pub fn new(page_image: Option<ImageData>, zoom: f64) -> Self {
        Self {
            page_image,
            transform: calculate_transform(zoom),
        }
    }

    /// Calculate transform based on zoom
    fn calculate_transform(zoom: f64) -> druid::kurbo::Affine {
        let scale = zoom as f64;
        druid::kurbo::Affine::scale(scale)
    }
}

impl Widget for PageContainer {
    fn event(
        &mut self,
        _event: &xilem::Event,
        _ctx: &mut xilem::EventCtx,
        _data: &mut crate::app_xilem::AppData,
        _env: &Env,
    ) -> bool {
        false
    }

    fn lifecycle(
        &mut self,
        _lifecycle: &xilem::LifeCycle,
        _ctx: &mut xilem::LifeCycleCtx,
        _data: &crate::app_xilem::AppData,
        _env: &Env,
    ) {
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &crate::app_xilem::AppData, env: &Env) {
        if let Some(image) = &self.page_image {
            ctx.draw_image(image, self.transform);
        } else {
            // Draw placeholder
            ctx.draw_text(
                &ctx.text("No page"),
                Vec2::new(0.0, 0.0),
                druid::PaintTextSettings::new(),
            );
        }
    }
}
