//! Menu bar widget

use druid::{
    widget::{Flex, Label, Menu, MenuItem, Row, UnitPoint},
    Color, Env, Widget,
};

/// Main menu bar
pub struct MenuBar {
    /// File menu
    file_menu: Menu<BoomagaApp>,
    /// View menu
    view_menu: Menu<BoomagaApp>,
    /// Print menu
    print_menu: Menu<BoomagaApp>,
    /// Help menu
    help_menu: Menu<BoomagaApp>,
}

impl MenuBar {
    /// Create a new menu bar
    pub fn new() -> Self {
        let app = BoomagaApp::new();

        Self {
            file_menu: Self::create_file_menu(),
            view_menu: Self::create_view_menu(),
            print_menu: Self::create_print_menu(),
            help_menu: Self::create_help_menu(),
        }
    }

    /// Create file menu
    fn create_file_menu() -> Menu<ViewerState> {
        Menu::new("File").add(
            MenuItem::new("Open Document").on_click(|_ctx, data, _env| {
                // TODO: Open document dialog
                tracing::info!("Open Document clicked");
            }),
        ).add(
            MenuItem::new("Print").on_click(|_ctx, data, _env| {
                // TODO: Print dialog
                tracing::info!("Print clicked");
            }),
        ).add(
            MenuItem::new("Print Settings").on_click(|_ctx, _data, _env| {
                tracing::info!("Print Settings clicked");
            }),
        ).add(
            MenuItem::new("Close").on_click(|_ctx, _data, _env| {
                tracing::info("Close clicked");
            }),
        )
    }

    /// Create view menu
    fn create_view_menu() -> Menu<ViewerState> {
        Menu::new("View").add(
            MenuItem::new("Previous Page").on_click(|_ctx, data, _env| {
                if data.current_page > 0 {
                    data.current_page -= 1;
                }
            }),
        ).add(
            MenuItem::new("Next Page").on_click(|_ctx, data, _env| {
                if let Some(document) = data.document.as_ref() {
                    if data.current_page < document.page_count() - 1 {
                        data.current_page += 1;
                    }
                }
            }),
        ).add(
            MenuItem::new("First Page").on_click(|_ctx, data, _env| {
                data.current_page = 0;
            }),
        ).add(
            MenuItem::new("Last Page").on_click(|_ctx, data, _env| {
                if let Some(document) = data.document.as_ref() {
                    data.current_page = document.page_count().saturating_sub(1);
                }
            }),
        ).separator().add(
            MenuItem::new("Zoom In").on_click(|_ctx, data, _env| {
                data.zoom_level = data.zoom_level * 1.2;
            }),
        ).add(
            MenuItem::new("Zoom Out").on_click(|_ctx, data, _env| {
                data.zoom_level = data.zoom_level / 1.2;
            }),
        ).add(
            MenuItem::new("Fit to Width").on_click(|_ctx, data, _env| {
                data.zoom_level = 1.0;
            }),
        )
    }

    /// Create print menu
    fn create_print_menu() -> Menu<ViewerState> {
        Menu::new("Print").add(
            MenuItem::new("Print Document").on_click(|_ctx, _data, _env| {
                tracing::info!("Print Document clicked");
            }),
        ).add(
            MenuItem::new("Cancel Job").on_click(|_ctx, _data, _env| {
                tracing::info("Cancel Job clicked");
            }),
        )
    }

    /// Create help menu
    fn create_help_menu() -> Menu<ViewerState> {
        Menu::new("Help").add(
            MenuItem::new("Documentation").on_click(|_ctx, _data, _env| {
                tracing::info("Documentation clicked");
            }),
        ).add(
            MenuItem::new("About").on_click(|_ctx, _data, _env| {
                tracing::info("About clicked");
            }),
        )
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<ViewerState> for MenuBar {
    fn event(&mut self, _ctx: &mut druid::EventCtx, _event: &druid::Event, _data: &mut ViewerState, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &ViewerState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &ViewerState, _data: &ViewerState, _env: &Env) {}

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &ViewerState, env: &Env) -> druid::Size {
        // Calculate height based on menu items
        let item_height = 24.0;
        let size = Size::new(bc.width(), item_height);

        ctx.constraints().constrain(size)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &ViewerState, env: &Env) {
        let size = ctx.size();

        // Draw menu bar background
        ctx.fill(
            Rect::ZERO.with_size(size),
            &Color::LIGHT_GRAY,
        );

        // Draw menu items
        // This is simplified - in production, use a proper menu implementation
        let items = vec![
            "File",
            "View",
            "Print",
            "Help",
        ];

        let item_width = size.width / items.len() as f64;
        let item_height = size.height;

        for (i, item) in items.iter().enumerate() {
            let x = i as f64 * item_width;
            ctx.fill(
                Rect::new(x, 0.0, item_width, item_height),
                &Color::WHITE,
            );

            ctx.draw_text(
                &druid::TextLayout::new(item)
                    .font("Sans")
                    .text_color(Color::BLACK)
                    .content_size()
                    .unwrap_or_default(),
                Point::new(x + 10.0, 4.0),
                &Color::BLACK,
            );
        }
    }
}
