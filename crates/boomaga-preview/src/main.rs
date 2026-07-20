//! Preview application for the Boomaga-IPP virtual printer.
//!
//! Native Wayland GUI built with **Xilem** (0.4). Phase C of the Druid→Xilem
//! migration (see `docs/XILEM_MIGRATION.md`) adds a Masonry PDF canvas to the
//! Phase B view tree. Document loading and background rendering follow in Phase D.

mod app;
mod document_renderer;
mod pdf_canvas;

use app::AppData;
use pdf_canvas::pdf_canvas;
use tracing::{info, Level};
use xilem::view::{button, flex, label, Axis};
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

/// The Xilem view tree, rebuilt from `AppData` on every state change.
fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> + use<> {
    let toolbar = flex(
        Axis::Horizontal,
        (
            button(label("⏮ First"), |d: &mut AppData| d.first_page()),
            button(label("◀ Previous"), |d: &mut AppData| d.previous_page()),
            button(label("Next ▶"), |d: &mut AppData| d.next_page()),
            button(label("Last ⏭"), |d: &mut AppData| d.last_page()),
            button(label("−"), |d: &mut AppData| d.zoom_out()),
            button(label("100%"), |d: &mut AppData| d.reset_zoom()),
            button(label("+"), |d: &mut AppData| d.zoom_in()),
        ),
    );

    let canvas = pdf_canvas(data.rendered_page.clone(), data.zoom);

    let status = if data.page_count() == 0 {
        format!("0 pages   ·   zoom {:.0}%", data.zoom * 100.0)
    } else {
        format!(
            "Page {} of {}   ·   zoom {:.0}%",
            data.current_page + 1,
            data.page_count(),
            data.zoom * 100.0
        )
    };

    flex(Axis::Vertical, (toolbar, canvas, label(status)))
}

fn main() -> anyhow::Result<()> {
    let debug = std::env::args().any(|a| a == "--debug");
    tracing_subscriber::fmt()
        .with_max_level(if debug { Level::DEBUG } else { Level::INFO })
        .with_target(false)
        .init();

    info!(
        "{} v{} starting (Xilem GUI)...",
        boomaga_core::constants::APP_NAME,
        boomaga_core::constants::APP_VERSION
    );

    // Single-window convenience: wraps `AppData` in `ExitOnClose` (→ `AppState`)
    // and the view returned by `app_logic` into one window.
    let app = Xilem::new_simple(
        AppData::default(),
        app_logic,
        WindowOptions::new(boomaga_core::constants::APP_NAME),
    );
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
