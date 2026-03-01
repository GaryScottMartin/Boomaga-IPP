//! Document rendering module using Poppler
//!
//! This module provides functionality to load, parse, and render PDF/PostScript
//! documents using the Poppler library.

use poppler::{Document as PopplerDocument, Page as PopplerPage};
use std::path::Path;
use std::sync::{Arc, Mutex};
use druid::kurbo::Rect;
use cairo::{ImageSurface, Context as CairoContext, Format};
use tracing::{info, warn, error};

use boomaga_core::{Document, Page, PageSize, Orientation, GraphicsElement, PathElement, Color};

/// Error types for document rendering
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Poppler error: {0}")]
    Poppler(#[from] poppler::Error),

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
    poppler_doc: Arc<Mutex<Option<PopplerDocument>>>,
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
        info!("Loading document: {:?}", path);

        // Open the PDF file using Poppler
        let doc = PopplerDocument::from_file(path, None)?;
        let doc_lock = self.poppler_doc.lock().map_err(|e| {
            error!("Failed to acquire lock: {}", e);
            RenderError::RenderFailed(0)
        })?;

        *doc_lock = Some(doc);

        // Parse metadata
        let mut core_doc = Document::new(self.doc_id.clone(), path.to_path_buf(), FileType::Pdf);

        // Try to get document information
        if let Some(info) = doc_lock.as_ref().unwrap().info() {
            if let Some(title) = info.title() {
                core_doc.title = title.to_string();
            }
            if let Some(author) = info.author() {
                core_doc.author = Some(author.to_string());
            }
            if let Some(creator) = info.creator() {
                core_doc.creator = Some(creator.to_string());
            }
            if let Some(subject) = info.subject() {
                core_doc.subject = Some(subject.to_string());
            }
        }

        // Parse pages
        let page_count = doc_lock.as_ref().unwrap().pages().len();
        info!("Document has {} pages", page_count);

        for (index, poppler_page) in doc_lock.as_ref().unwrap().pages().iter().enumerate() {
            // Parse page properties
            let (width, height) = poppler_page.size();
            let orientation = poppler_page.orientation();

            // Convert to boomaga-core types
            let page_size = self.convert_page_size(width, height);
            let page_orientation = self.convert_orientation(orientation);

            // Create page object
            let mut page = Page::new(index + 1, width, height, page_orientation);

            // Extract page contents
            let elements = self.extract_page_contents(poppler_page);
            page.contents = boomaga_core::PageContents::Vector(elements);

            core_doc.add_page(page);
        }

        // Parse metadata
        core_doc.parse_metadata().await?;

        Ok(core_doc)
    }

    /// Render a page to a Cairo image surface
    ///
    /// Returns an ImageSurface with the rendered page
    pub fn render_page_to_surface(
        &self,
        page_number: usize,
        width_points: f64,
        height_points: f64,
        dpi: f64,
    ) -> Result<ImageSurface, RenderError> {
        let doc_lock = self.poppler_doc.lock().map_err(|e| {
            error!("Failed to acquire lock: {}", e);
            RenderError::RenderFailed(page_number)
        })?;

        let doc = doc_lock.as_ref().ok_or(RenderError::EmptyDocument)?;

        let poppler_page = doc
            .page(page_number)
            .ok_or(RenderError::RenderFailed(page_number))?;

        // Convert points to pixels at given DPI
        let width_px = (width_points * dpi / 72.0) as i32;
        let height_px = (height_points * dpi / 72.0) as i32;

        // Create image surface
        let surface = ImageSurface::create(Format::ARgb32, width_px, height_px)
            .map_err(|e| RenderError::RenderFailed(page_number))?;

        let cairo_ctx = CairoContext::new(&surface);
        cairo_ctx.scale(dpi / 72.0, dpi / 72.0);

        // Render page
        let error = poppler_page.render(&cairo_ctx);
        if error != poppler::RenderError::Success {
            return Err(RenderError::RenderFailed(page_number));
        }

        Ok(surface)
    }

    /// Extract graphics elements from a poppler page
    fn extract_page_contents(&self, poppler_page: &PopplerPage) -> Vec<GraphicsElement> {
        let mut elements = Vec::new();

        // Extract text elements
        if let Some(text) = poppler_page.text() {
            if !text.is_empty() {
                elements.push(GraphicsElement::Text {
                    content: text.to_string(),
                    font: "Times-Roman".to_string(),
                    size: 12.0,
                    x: 50.0,
                    y: 750.0,
                    color: Color::black(),
                });
            }
        }

        // Extract form fields (if any)
        if let Some(annots) = poppler_page.annots() {
            for annot in annots {
                if let Some(rect) = annot.rect() {
                    elements.push(GraphicsElement::Rectangle {
                        x: rect.x0,
                        y: rect.y0,
                        width: rect.x1 - rect.x0,
                        height: rect.y1 - rect.y0,
                        fill: None,
                        stroke: Some(Color::black()),
                        stroke_width: 1.0,
                    });
                }
            }
        }

        elements
    }

    /// Convert Poppler page size to boomaga PageSize
    fn convert_page_size(&self, width: f64, height: f64) -> PageSize {
        // Compare with standard sizes
        let a4_width_mm = 210.0;
        let a4_height_mm = 297.0;
        let letter_width_mm = 215.9;
        let letter_height_mm = 279.4;

        let pdf_width_mm = width * 25.4 / 72.0;
        let pdf_height_mm = height * 25.4 / 72.0;

        // Check if close to A4 (within 5% tolerance)
        if self.is_close_to(pdf_width_mm, a4_width_mm) && self.is_close_to(pdf_height_mm, a4_height_mm) {
            return PageSize::A4;
        }

        // Check if close to Letter
        if self.is_close_to(pdf_width_mm, letter_width_mm) && self.is_close_to(pdf_height_mm, letter_height_mm) {
            return PageSize::Letter;
        }

        // Otherwise, create custom size
        PageSize::Custom { width, height }
    }

    /// Check if two values are close within tolerance
    fn is_close_to(&self, value: f64, target: f64) -> bool {
        let tolerance = target * 0.05; // 5% tolerance
        (value - target).abs() < tolerance
    }

    /// Convert Poppler orientation to boomaga Orientation
    fn convert_orientation(&self, orientation: poppler::PageOrientation) -> Orientation {
        match orientation {
            poppler::PageOrientation::Portrait => Orientation::Portrait,
            poppler::PageOrientation::Landscape => Orientation::Landscape,
            poppler::PageOrientation::UpsideDownPortrait => Orientation::UpsideDownPortrait,
            poppler::PageOrientation::UpsideDownLandscape => Orientation::UpsideDownLandscape,
        }
    }
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Pdf,
    PostScript,
    Unknown,
}

impl FileType {
    /// Determine file type from extension
    pub fn from_path(path: &Path) -> Self {
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .to_lowercase();

        match extension.as_str() {
            "pdf" => FileType::Pdf,
            "ps" | "eps" => FileType::PostScript,
            _ => FileType::Unknown,
        }
    }

    /// Get MIME type for the file
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileType::Pdf => "application/pdf",
            FileType::PostScript => "application/postscript",
            FileType::Unknown => "application/octet-stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(FileType::from_path(Path::new("test.pdf")), FileType::Pdf);
        assert_eq!(FileType::from_path(Path::new("test.PDF")), FileType::Pdf);
        assert_eq!(FileType::from_path(Path::new("test.ps")), FileType::PostScript);
        assert_eq!(FileType::from_path(Path::new("test.eps")), FileType::PostScript);
        assert_eq!(FileType::from_path(Path::new("test.txt")), FileType::Unknown);
    }
}
