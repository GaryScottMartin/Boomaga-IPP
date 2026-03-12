//! Main document viewer widget for Xilem

use xilem::{Widget, Flex, Label, Event, EventCtx, PaintCtx, Env};
use crate::app_xilem::AppData;
use crate::widgets::page_container::PageContainer;

/// Document viewer widget
pub struct DocumentViewer {
    page_container: PageContainer,
    page_info: Label,
    navigation_buttons: Flex,
    zoom_controls: Flex,
}

impl DocumentViewer {
    /// Create a new document viewer
    pub fn new() -> Self {
        Self {
            page_container: PageContainer::new(None, 1.0),
            page_info: Label::new("Page 0 of 0"),
            navigation_buttons: Flex::row(),
            zoom_controls: Flex::row(),
        }
    }

    /// Handle navigation events
    fn handle_navigation(&mut self, event: &Event, data: &mut AppData, ctx: &mut EventCtx) -> bool {
        match event {
            Event::Click(button) => match button {
                NavigationButton::Prev => {
                    data.previous_page();
                    ctx.request_paint();
                    true
                }
                NavigationButton::Next => {
                    data.next_page();
                    ctx.request_paint();
                    true
                }
                ZoomButton::In => {
                    data.set_zoom(data.zoom * 1.2);
                    ctx.request_paint();
                    true
                }
                ZoomButton::Out => {
                    data.set_zoom(data.zoom / 1.2);
                    ctx.request_paint();
                    true
                }
            },
            _ => false,
        }
    }
}

/// Navigation button type
pub enum NavigationButton {
    Prev,
    Next,
}

/// Zoom button type
pub enum ZoomButton {
    In,
    Out,
}

impl Widget for DocumentViewer {
    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut AppData,
        env: &Env,
    ) -> bool {
        self.handle_navigation(event, data, ctx)
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
        // Draw document viewer layout
        let layout = Flex::column()
            .with_child(self.page_container)
            .with_spacing(8.0)
            .with_child(self.page_info.clone());

        layout.paint(ctx, data, env);
    }
}
