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
    fn create_file_menu() -> Menu<BoomagaApp> {
        Menu::new("File").add(
            MenuItem::new("Open Document").on_click(|_ctx, data, _env| {
                // TODO: Open document dialog
                tracing::info!("Open Document clicked");
            }),
        ).add(
            MenuItem::new("Print").on_click(|_ctx, data, _env| {
                data.open_print_dialog();
            }),
        ).add(
            MenuItem::new("Print Settings").on_click(|_ctx, data, _env| {
                data.open_settings();
            }),
        ).add(
            MenuItem::new("Close").on_click(|_ctx, _data, _env| {
                tracing::info!("Close clicked");
            }),
        )
    }

    /// Create view menu
    fn create_view_menu() -> Menu<BoomagaApp> {
        Menu::new("View").add(
            MenuItem::new("Previous Page").on_click(|_ctx, data, _env| {
                data.previous_page();
            }),
        ).add(
            MenuItem::new("Next Page").on_click(|_ctx, data, _env| {
                data.next_page();
            }),
        ).add(
            MenuItem::new("First Page").on_click(|_ctx, data, _env| {
                data.first_page();
            }),
        ).add(
            MenuItem::new("Last Page").on_click(|_ctx, data, _env| {
                data.last_page();
            }),
        ).separator().add(
            MenuItem::new("Zoom In").on_click(|_ctx, data, _env| {
                data.set_zoom(data.zoom_level * 1.2);
            }),
        ).add(
            MenuItem::new("Zoom Out").on_click(|_ctx, data, _env| {
                data.set_zoom(data.zoom_level / 1.2);
            }),
        ).add(
            MenuItem::new("Fit to Width").on_click(|_ctx, data, _env| {
                data.set_zoom(1.0);
            }),
        )
    }

    /// Create print menu
    fn create_print_menu() -> Menu<BoomagaApp> {
        Menu::new("Print").add(
            MenuItem::new("Print Document").on_click(|_ctx, data, _env| {
                data.print_document();
            }),
        ).add(
            MenuItem::new("Cancel Job").on_click(|_ctx, data, _env| {
                data.cancel_job();
            }),
        )
    }

    /// Create help menu
    fn create_help_menu() -> Menu<BoomagaApp> {
        Menu::new("Help").add(
            MenuItem::new("Documentation").on_click(|_ctx, _data, _env| {
                tracing::info!("Documentation clicked");
            }),
        ).add(
            MenuItem::new("About").on_click(|_ctx, _data, _env| {
                tracing::info!("About clicked");
            }),
        )
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<BoomagaApp> for MenuBar {
    fn event(&mut self, _ctx: &mut druid::EventCtx, _event: &druid::Event, _data: &mut BoomagaApp, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &BoomagaApp, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &BoomagaApp, _data: &BoomagaApp, _env: &Env) {}

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &BoomagaApp, env: &Env) -> druid::Size {
        // Calculate height based on menu items
        let item_height = 24.0;
        let size = Size::new(bc.width(), item_height);

        ctx.constraints().constrain(size)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &BoomagaApp, env: &Env) {
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
