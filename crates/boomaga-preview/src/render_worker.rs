//! Xilem worker bridge for file selection and thread-confined PDF rendering.

use std::fmt;
use std::path::PathBuf;

use boomaga_core::Document;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use xilem::core::{MessageProxy, View};
use xilem::view::worker;
use xilem::ViewCtx;

use crate::app::AppData;
use crate::document_renderer::DocumentRenderer;
use crate::pdf_canvas::CanvasImage;

const PREVIEW_DPI: f64 = 96.0;

/// Commands sent from the UI state to the renderer thread.
#[derive(Debug)]
pub enum RendererCommand {
    OpenFileDialog,
    Load {
        generation: u64,
        path: PathBuf,
    },
    RenderPage {
        generation: u64,
        page_index: usize,
    },
}

/// Results delivered to `AppData` on Xilem's UI thread.
pub enum RendererEvent {
    FileSelected(PathBuf),
    FileDialogCancelled,
    DocumentLoaded {
        generation: u64,
        path: PathBuf,
        document: Document,
    },
    PageRendered {
        generation: u64,
        page_index: usize,
        image: CanvasImage,
    },
    Failed {
        generation: Option<u64>,
        page_index: Option<usize>,
        message: String,
    },
}

impl fmt::Debug for RendererEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileSelected(path) => formatter.debug_tuple("FileSelected").field(path).finish(),
            Self::FileDialogCancelled => formatter.write_str("FileDialogCancelled"),
            Self::DocumentLoaded {
                generation,
                path,
                document,
            } => formatter
                .debug_struct("DocumentLoaded")
                .field("generation", generation)
                .field("path", path)
                .field("page_count", &document.page_count())
                .finish(),
            Self::PageRendered {
                generation,
                page_index,
                ..
            } => formatter
                .debug_struct("PageRendered")
                .field("generation", generation)
                .field("page_index", page_index)
                .finish_non_exhaustive(),
            Self::Failed {
                generation,
                page_index,
                message,
            } => formatter
                .debug_struct("Failed")
                .field("generation", generation)
                .field("page_index", page_index)
                .field("message", message)
                .finish(),
        }
    }
}

/// A persistent Xilem worker which owns the renderer command channel.
pub fn renderer_worker() -> impl View<AppData, (), ViewCtx> {
    worker(
        run_renderer_thread,
        |data: &mut AppData, sender| data.install_renderer(sender),
        |data: &mut AppData, event| data.handle_renderer_event(event),
    )
}

async fn run_renderer_thread(
    proxy: MessageProxy<RendererEvent>,
    receiver: UnboundedReceiver<RendererCommand>,
) {
    let failure_proxy = proxy.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("boomaga-pdf-renderer".to_owned())
        .spawn(move || renderer_loop(proxy, receiver))
    {
        let _ = failure_proxy.message(RendererEvent::Failed {
            generation: None,
            page_index: None,
            message: format!("failed to start PDF renderer: {error}"),
        });
    }
}

fn renderer_loop(
    proxy: MessageProxy<RendererEvent>,
    mut receiver: UnboundedReceiver<RendererCommand>,
) {
    let mut active_generation = None;
    let mut renderer = None;

    while let Some(command) = receiver.blocking_recv() {
        let event = match command {
            RendererCommand::OpenFileDialog => {
                let selected = rfd::FileDialog::new()
                    .set_title("Open PDF")
                    .add_filter("PDF documents", &["pdf"])
                    .pick_file();
                selected.map_or(RendererEvent::FileDialogCancelled, RendererEvent::FileSelected)
            }
            RendererCommand::Load { generation, path } => {
                let mut next_renderer = DocumentRenderer::new(path.to_string_lossy());
                match next_renderer.load(&path) {
                    Ok(document) => {
                        active_generation = Some(generation);
                        renderer = Some(next_renderer);
                        RendererEvent::DocumentLoaded {
                            generation,
                            path,
                            document,
                        }
                    }
                    Err(error) => RendererEvent::Failed {
                        generation: Some(generation),
                        page_index: None,
                        message: error.to_string(),
                    },
                }
            }
            RendererCommand::RenderPage {
                generation,
                page_index,
            } => {
                if active_generation != Some(generation) {
                    continue;
                }
                let Some(active_renderer) = renderer.as_ref() else {
                    continue;
                };
                match active_renderer.render_page(page_index, PREVIEW_DPI) {
                    Ok(image) => RendererEvent::PageRendered {
                        generation,
                        page_index,
                        image,
                    },
                    Err(error) => RendererEvent::Failed {
                        generation: Some(generation),
                        page_index: Some(page_index),
                        message: error.to_string(),
                    },
                }
            }
        };

        if proxy.message(event).is_err() {
            break;
        }
    }
}

pub type RendererSender = UnboundedSender<RendererCommand>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_commands_and_events_are_send() {
        fn assert_send<T: Send>() {}
        assert_send::<RendererCommand>();
        assert_send::<RendererEvent>();
    }
}
