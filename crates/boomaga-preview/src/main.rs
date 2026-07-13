//! Preview application for the Boomaga-IPP virtual printer.
//!
//! Native Wayland GUI built with **Xilem** (0.4). This is the Phase-A skeleton
//! of the Druid→Xilem migration (see `docs/XILEM_MIGRATION.md`): a compiling
//! Xilem app with state (`AppData`) and a minimal reactive view tree
//! (navigation + zoom + status). Document rendering (`document_renderer.rs`,
//! currently dormant), imposition, and the IPC/print wiring come in later phases.

mod app;
// `document_renderer` (poppler + cairo) is retained on disk but not yet wired
// into the view tree — it is re-introduced in Phase C (needs a `poppler::{Document,
// Page}` vs `boomaga_core::{Document, Page}` import-collision fix). Keeping it
// un-`mod`-ded keeps the Phase-A skeleton minimal and compiling.
// mod document_renderer;

use app::AppData;
use tracing::{info, Level};
use xilem::view::{button, flex, label};
use xilem::{EventLoop, WidgetView, Xilem};

/// The Xilem view tree, rebuilt from `AppData` on every state change.
fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> + use<> {
    let status = if data.page_count() == 0 {
        format!("No document loaded   ·   zoom {:.0}%", data.zoom * 100.0)
    } else {
        format!(
            "page {} / {}   ·   zoom {:.0}%",
            data.current_page + 1,
            data.page_count(),
            data.zoom * 100.0
        )
    };

    flex((
        label(status),
        button("|< first", |d: &mut AppData| d.first_page()),
        button("< prev", |d: &mut AppData| d.previous_page()),
        button("next >", |d: &mut AppData| d.next_page()),
        button("last >|", |d: &mut AppData| d.last_page()),
        button("zoom -", |d: &mut AppData| d.zoom_out()),
        button("100%", |d: &mut AppData| d.reset_zoom()),
        button("zoom +", |d: &mut AppData| d.zoom_in()),
    ))
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

    let app = Xilem::new(AppData::default(), app_logic);
    app.run_windowed(EventLoop::with_user_event(), boomaga_core::constants::APP_NAME.into())?;
    Ok(())
}
