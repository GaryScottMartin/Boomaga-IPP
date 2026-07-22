//! Preview application for the Boomaga-IPP virtual printer.
//!
//! Native Wayland GUI built with **Xilem** (0.4). Phase D adds native PDF
//! selection and moves Poppler loading and on-demand page rendering off the UI
//! thread through Xilem's worker/message mechanism.

mod app;
mod document_renderer;
mod pdf_canvas;
mod render_worker;

use app::{AppData, FillOrder, LoadState};
use boomaga_core::PagesPerSheet;
use pdf_canvas::pdf_canvas;
use render_worker::renderer_worker;
use std::ffi::OsStr;
use std::path::PathBuf;
use tracing::{info, Level};
use xilem::core::fork;
use xilem::masonry::properties::types::AsUnit;
use xilem::style::Style as _;
use xilem::view::{button, flex, label, sized_box, Axis, FlexExt as _, FlexSpacer};
use xilem::{Color, EventLoop, WidgetView, WindowOptions, Xilem};

/// The Xilem view tree, rebuilt from `AppData` on every state change.
fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> + use<> {
    let toolbar = flex(
        Axis::Horizontal,
        (
            button(label("Open PDF…"), |d: &mut AppData| d.choose_document()),
            button(label("−"), |d: &mut AppData| d.zoom_out()),
            button(label("100%"), |d: &mut AppData| d.reset_zoom()),
            button(label("+"), |d: &mut AppData| d.zoom_in()),
        ),
    );

    let imposition_toolbar = flex(
        Axis::Horizontal,
        (
            button(label("1-up"), |d: &mut AppData| {
                d.set_pages_per_sheet(PagesPerSheet::One)
            }),
            button(label("2-up"), |d: &mut AppData| {
                d.set_pages_per_sheet(PagesPerSheet::Two)
            }),
            button(label("4-up"), |d: &mut AppData| {
                d.set_pages_per_sheet(PagesPerSheet::Four)
            }),
            button(label("6-up"), |d: &mut AppData| {
                d.set_pages_per_sheet(PagesPerSheet::Six)
            }),
            button(label("8-up"), |d: &mut AppData| {
                d.set_pages_per_sheet(PagesPerSheet::Eight)
            }),
            button(label("Horizontal"), |d: &mut AppData| {
                d.set_fill_order(FillOrder::Horizontal)
            }),
            button(label("Vertical"), |d: &mut AppData| {
                d.set_fill_order(FillOrder::Vertical)
            }),
        ),
    );

    let canvas = pdf_canvas(
        data.current_canvas_images(),
        data.print_options.pages_per_sheet as u8,
        data.fill_order == FillOrder::Vertical,
        data.zoom,
    );
    let status = status_text(data);
    let footer = sized_box(flex(
        Axis::Horizontal,
        (
            button(label("⏮ First"), |d: &mut AppData| d.first_page()),
            button(label("◀ Previous"), |d: &mut AppData| d.previous_page()),
            button(label("Next ▶"), |d: &mut AppData| d.next_page()),
            button(label("Last ⏭"), |d: &mut AppData| d.last_page()),
            FlexSpacer::Flex(1.0),
            label(status),
        ),
    ))
    .expand_width()
    .height(32.px())
    .border(Color::from_rgb8(96, 96, 96), 1.0);
    let content = sized_box(
        flex(
            Axis::Vertical,
            (toolbar, imposition_toolbar, canvas.flex(1.0)),
        )
        .must_fill_major_axis(true),
    )
    .expand_height();
    let interface = flex(Axis::Vertical, (content.flex(1.0), footer)).must_fill_major_axis(true);

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
                "Sheet {} of {page_count} ({page_status})   ·   {}-up   ·   cached {rendered}/{}   ·   zoom {:.0}%",
                data.current_page + 1,
                data.print_options.pages_per_sheet as u8,
                data.rendered_pages.len(),
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
