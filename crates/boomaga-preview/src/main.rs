//! Preview application for the Boomaga-IPP virtual printer.
//!
//! Native Wayland GUI built with **Xilem** (0.4). Phase D adds native PDF
//! selection and moves Poppler loading and on-demand page rendering off the UI
//! thread through Xilem's worker/message mechanism.

mod app;
mod document_renderer;
mod pdf_canvas;
mod render_worker;

use app::{AppData, LoadState};
use pdf_canvas::pdf_canvas;
use render_worker::renderer_worker;
use std::ffi::OsStr;
use std::path::PathBuf;
use tracing::{info, Level};
use xilem::core::fork;
use xilem::view::{button, flex, label, Axis};
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

/// The Xilem view tree, rebuilt from `AppData` on every state change.
fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> + use<> {
    let toolbar = flex(
        Axis::Horizontal,
        (
            button(label("Open PDF…"), |d: &mut AppData| d.choose_document()),
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
    let status = status_text(data);
    let interface = flex(Axis::Vertical, (toolbar, canvas, label(status)));

    fork(interface, renderer_worker())
}

fn status_text(data: &AppData) -> String {
    if data.choosing_file {
        return "Selecting a PDF…".to_owned();
    }

    if let Some(error) = &data.error_message {
        return format!("Error: {error}");
    }

    match data.load_state {
        LoadState::Idle => "No PDF open".to_owned(),
        LoadState::Loading => data.document_path.as_ref().map_or_else(
            || "Loading PDF…".to_owned(),
            |path| format!("Loading {}…", path.display()),
        ),
        LoadState::Error => "Unable to load PDF".to_owned(),
        LoadState::Ready => {
            let page_count = data.page_count();
            let rendered = data.rendered_page_count();
            let page_status = if data.current_canvas_image().is_some() {
                "ready"
            } else {
                "rendering"
            };
            format!(
                "Page {} of {page_count} ({page_status})   ·   cached {rendered}/{page_count}   ·   zoom {:.0}%",
                data.current_page + 1,
                data.zoom * 100.0
            )
        }
    }
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

    let initial_state = document_path.map_or_else(AppData::default, AppData::with_document_path);
    let app = Xilem::new_simple(
        initial_state,
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
