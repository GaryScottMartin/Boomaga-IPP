//! Toolbar widget

use druid::{
    widget::{Flex, Label, UnitPoint},
    Color, Env, Widget,
};

/// Toolbar widget
pub struct Toolbar {
    /// Toolbar items
    items: Vec<ToolbarItem>,
}

/// Toolbar item type
#[derive(Debug, Clone)]
pub enum ToolbarItem {
    /// Navigation button
    Navigation { label: String, enabled: bool },
    /// Zoom button
    Zoom { label: String, enabled: bool },
    /// Print button
    Print { enabled: bool },
    /// Cancel button
    Cancel { enabled: bool },
}

impl Toolbar {
    /// Create a new toolbar
    pub fn new() -> Self {
        Self {
            items: vec![
                ToolbarItem::Navigation {
                    label: "Previous".to_string(),
                    enabled: true,
                },
                ToolbarItem::Navigation {
                    label: "Next".to_string(),
                    enabled: true,
                },
                ToolbarItem::Zoom {
                    label: "100%".to_string(),
                    enabled: true,
                },
                ToolbarItem::Print {
                    enabled: false,
                },
                ToolbarItem::Cancel {
                    enabled: false,
                },
            ],
        }
    }

    /// Update toolbar state
    pub fn update_state(&mut self, document_loaded: bool, job_status: Option<String>) {
        // Update navigation buttons
        if let ToolbarItem::Navigation { label, enabled } = &mut self.items[0] {
            *enabled = true;
        }
        if let ToolbarItem::Navigation { label, enabled } = &mut self.items[1] {
            *enabled = true;
        }

        // Update zoom button
        if let ToolbarItem::Zoom { label, enabled } = &mut self.items[2] {
            *enabled = true;
        }

        // Update print button
        if let ToolbarItem::Print { enabled } = &mut self.items[3] {
            *enabled = document_loaded;
        }

        // Update cancel button
        if let ToolbarItem::Cancel { enabled } = &mut self.items[4] {
            *enabled = job_status.is_some();
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<BoomagaApp> for Toolbar {
    fn event(&mut self, _ctx: &mut druid::EventCtx, _event: &druid::Event, _data: &BoomagaApp, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &BoomagaApp, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &BoomagaApp, _data: &BoomagaApp, _env: &Env) {}

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &BoomagaApp, env: &Env) -> druid::Size {
        // Calculate toolbar height
        let toolbar_height = 40.0;
        let size = Size::new(bc.width(), toolbar_height);

        ctx.constraints().constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &BoomagaApp, _env: &Env) {
        let size = ctx.size();

        // Draw toolbar background
        ctx.fill(
            Rect::ZERO.with_size(size),
            &Color::LIGHT_GRAY,
        );

        // Draw toolbar separator
        ctx.stroke_line(
            (0.0, size.height),
            (size.width, size.height),
            &Color::GRAY300,
        );
    }
}
