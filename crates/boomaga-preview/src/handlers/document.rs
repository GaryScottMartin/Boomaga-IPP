//! Document event handlers for Xilem

use crate::app_xilem::AppData;
use xilem::{Event, EventCtx, WindowCtx};
use std::path::PathBuf;

/// Handle document loading
pub fn handle_open_document(ctx: &mut EventCtx, data: &mut AppData) {
    // TODO: Implement file dialog
    let path = PathBuf::from("/tmp/test.pdf");
    if path.exists() {
        // Load document
        data.document_path = Some(path.clone());
        data.current_page = 0;
        data.zoom = 1.0;
    }
}

/// Handle print action
pub fn handle_print(_ctx: &mut EventCtx, data: &AppData) {
    // TODO: Implement print dialog
    println!("Printing document: {:?}", data.document_path);
}

/// Handle file menu actions
pub fn handle_file_menu(action: FileMenuAction, ctx: &mut EventCtx, data: &mut AppData) {
    match action {
        FileMenuAction::OpenDocument => handle_open_document(ctx, data),
        FileMenuAction::Print => handle_print(ctx, data),
        FileMenuAction::Exit => ctx.close_window(),
    }
}

/// File menu action type
pub enum FileMenuAction {
    OpenDocument,
    Print,
    Exit,
}
