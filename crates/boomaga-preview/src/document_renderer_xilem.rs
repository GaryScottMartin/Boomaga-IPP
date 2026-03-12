//! Xilem-compatible document renderer

use poppler::{Document, Page};
use boomaga_core::{Document, Page as CorePage, PageContents, FileType};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Error types for document rendering
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Poppler error: {0}")]
    Poppler(String),

    #[error("Invalid document path")]
    InvalidPath,

    #[error("Document is empty")]
    EmptyDocument,

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Failed to render page {0}")]
    RenderFailed(usize),
}

/// Document renderer that loads and renders PDF/PostScript files
pub struct DocumentRenderer {
    poppler_doc: Arc<Mutex<Option<Document>>>,
    doc_id: String,
}

impl DocumentRenderer {
    /// Create a new document renderer
    pub fn new(doc_id: String) -> Self {
        Self {
            poppler_doc: Arc::new(Mutex::new(None)),
            doc_id,
        }
    }

    /// Load a document from file path
    pub fn load(&self, path: &Path) -> Result<Document, RenderError> {
        let doc = Document::from_file(path, None)?;
        let doc_lock = self.poppler_doc.lock().map_err(|e| {
            RenderError::Poppler(e.to_string())
        })?;

        *doc_lock = Some(doc);

        // Create boomaga_core::Document
        let mut core_doc = Document::new(self.doc_id.clone(), path.to_path_buf(), FileType::Pdf);

        // Parse pages
        let page_count = doc_lock.as_ref().unwrap().pages().len();

        for (index, poppler_page) in doc_lock.as_ref().unwrap().pages().iter().enumerate() {
            let (width, height) = poppler_page.size();

            // Create page object
            let mut page = CorePage::new(index + 1, width, height, boomaga_core::Orientation::Portrait);

            // Extract page contents
            let elements = Vec::new(); // TODO: Implement content extraction
            page.contents = PageContents::Vector(elements);

            core_doc.add_page(page);
        }

        Ok(core_doc)
    }
}
