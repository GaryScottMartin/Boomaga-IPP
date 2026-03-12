//! Xilem-based preview application

mod app_xilem;
mod window;
mod document_renderer_xilem;
mod widgets;
mod handlers;

use tracing::{info, error, Level};
use std::env;
use xilem::{App, Color, Event, EventCtx, LifeCycle, LifeCycleCtx, PaintCtx, Env, Widget};
use crate::app_xilem::AppData;
use crate::widgets::DocumentViewer;

/// Xilem application widget
struct BoomagaWidget {
    viewer: DocumentViewer,
}

impl BoomagaWidget {
    fn new() -> Self {
        Self {
            viewer: DocumentViewer::new(),
        }
    }
}

impl Widget for BoomagaWidget {
    fn event(
        &mut self,
        event: &Event,
        ctx: &mut EventCtx,
        data: &mut AppData,
        env: &Env,
    ) -> bool {
        self.viewer.event(event, ctx, data, env)
    }

    fn lifecycle(
        &mut self,
        lifecycle: &LifeCycle,
        ctx: &mut LifeCycleCtx,
        data: &AppData,
        env: &Env,
    ) {
        self.viewer.lifecycle(lifecycle, ctx, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        // Draw background
        ctx.fill(
            ctx.size().to_rect(),
            &Color::rgb8(240, 240, 240),
        );

        // Draw document viewer
        self.viewer.paint(ctx, data, env);
    }
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Initialize logging
    let log_level = if args.contains(&"--debug".to_string()) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("{} v{} starting...", boomaga_core::constants::APP_NAME, boomaga_core::constants::APP_VERSION);

    // Create window and run application
    let options = window::create_window();

    if let Err(e) = App::launch(BoomagaWidget::new(), options) {
        error!("Application error: {}", e);
        return Err(e);
    }

    Ok(())
}
