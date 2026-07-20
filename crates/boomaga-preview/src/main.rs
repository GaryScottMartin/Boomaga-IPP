//! Preview application for the Boomaga-IPP virtual printer.
//!
//! Native Wayland GUI built with **Xilem** (0.4). Phase C of the Druid→Xilem
//! migration (see `docs/XILEM_MIGRATION.md`) adds a Masonry PDF canvas to the
//! Phase B view tree. Phase C can synchronously load a PDF supplied on the
//! command line; background rendering follows in Phase D.

mod app;
mod document_renderer;
mod pdf_canvas;

use app::AppData;
use document_renderer::DocumentRenderer;
use pdf_canvas::pdf_canvas;
use std::ffi::OsStr;
use std::path::PathBuf;
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

    let canvas = pdf_canvas(data.current_canvas_image().cloned(), data.zoom);

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
    let (debug, document_path) = parse_args()?;
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
        load_initial_state(document_path)?,
        app_logic,
        WindowOptions::new(boomaga_core::constants::APP_NAME),
    );
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}

fn parse_args() -> anyhow::Result<(bool, Option<PathBuf>)> {
    let mut debug = false;
    let mut document_path = None;

    for argument in std::env::args_os().skip(1) {
        if argument == OsStr::new("--debug") {
            debug = true;
        } else if document_path.is_none() {
            document_path = Some(PathBuf::from(argument));
        } else {
            anyhow::bail!("expected at most one PDF path");
        }
    }

    Ok((debug, document_path))
}

fn load_initial_state(document_path: Option<PathBuf>) -> anyhow::Result<AppData> {
    let Some(path) = document_path else {
        return Ok(AppData::default());
    };

    let mut renderer = DocumentRenderer::new(path.to_string_lossy());
    let document = renderer.load(&path)?;
    let rendered_pages = (0..document.page_count())
        .map(|page_index| renderer.render_page(page_index, 96.0))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AppData {
        document_path: Some(path),
        document: Some(document),
        rendered_pages,
        ..AppData::default()
    })
}
