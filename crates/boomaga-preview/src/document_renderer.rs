//! Thread-confined PDF loading and page rendering with Poppler and Cairo.
//!
//! Poppler documents are not `Send` or `Sync`. Phase D's renderer worker creates
//! and retains this type on one dedicated thread; only core document metadata
//! and completed canvas images cross back to Xilem's UI thread.

use std::path::Path;

use boomaga_core::{
    Color, Document as CoreDocument, FileType, GraphicsElement, Orientation, Page as CorePage,
    PageContents,
};
use cairo::{Context, Format, ImageSurface};
use poppler::{PopplerDocument, PopplerPage};
use tracing::info;

use crate::pdf_canvas::{CanvasImage, CanvasImageError};

/// Failures while loading or rasterizing a PDF document.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Poppler error: {0}")]
    Poppler(String),

    #[error("Cairo error: {0}")]
    Cairo(#[from] cairo::Error),

    #[error("document contains no pages")]
    EmptyDocument,

    #[error("page {0} does not exist")]
    InvalidPage(usize),

    #[error("DPI must be finite and greater than zero")]
    InvalidDpi,

    #[error("rendered page dimensions exceed Cairo limits")]
    InvalidDimensions,

    #[error("failed to borrow rendered Cairo pixels: {0}")]
    SurfaceData(String),

    #[error(transparent)]
    CanvasImage(#[from] CanvasImageError),
}

/// Owns one Poppler document and renders its pages synchronously.
pub struct DocumentRenderer {
    poppler_document: Option<PopplerDocument>,
    document_id: String,
}

impl DocumentRenderer {
    /// Create an empty renderer for the given application document id.
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            poppler_document: None,
            document_id: document_id.into(),
        }
    }

    /// Load a PDF and build the framework-independent document model.
    pub fn load(&mut self, path: &Path) -> Result<CoreDocument, RenderError> {
        info!(path = ?path, "loading PDF document");

        let poppler_document = PopplerDocument::new_from_file(path, None)
            .map_err(|error| RenderError::Poppler(error.to_string()))?;
        if poppler_document.is_empty() {
            return Err(RenderError::EmptyDocument);
        }

        let mut document =
            CoreDocument::new(self.document_id.clone(), path.to_path_buf(), FileType::Pdf);
        if let Some(title) = poppler_document.get_title() {
            document.title = title;
        }

        for (index, poppler_page) in poppler_document.pages().enumerate() {
            let (width, height) = poppler_page.get_size();
            let orientation = page_orientation(width, height);
            let mut page = CorePage::new(index, width, height, orientation);
            page.contents = PageContents::Vector(extract_page_contents(&poppler_page));
            document.add_page(page);
        }

        info!(pages = document.page_count(), "loaded PDF document");
        self.poppler_document = Some(poppler_document);
        Ok(document)
    }

    /// Render a zero-based page index directly into a Masonry canvas image.
    pub fn render_page(&self, page_index: usize, dpi: f64) -> Result<CanvasImage, RenderError> {
        let mut surface = self.render_page_to_surface(page_index, dpi)?;
        surface.flush();

        let width = u32::try_from(surface.width()).map_err(|_| RenderError::InvalidDimensions)?;
        let height = u32::try_from(surface.height()).map_err(|_| RenderError::InvalidDimensions)?;
        let stride =
            usize::try_from(surface.stride()).map_err(|_| RenderError::InvalidDimensions)?;
        let row_bytes = width as usize * 4;
        let pixels = {
            let data = surface
                .data()
                .map_err(|error| RenderError::SurfaceData(error.to_string()))?;
            let mut pixels = Vec::with_capacity(row_bytes * height as usize);
            for row in data.chunks(stride).take(height as usize) {
                pixels.extend_from_slice(&row[..row_bytes]);
            }
            pixels
        };

        CanvasImage::from_cairo_bgra(pixels, width, height).map_err(Into::into)
    }

    /// Render a zero-based page index to a Cairo ARGB32 image surface.
    pub fn render_page_to_surface(
        &self,
        page_index: usize,
        dpi: f64,
    ) -> Result<ImageSurface, RenderError> {
        if !dpi.is_finite() || dpi <= 0.0 {
            return Err(RenderError::InvalidDpi);
        }

        let document = self
            .poppler_document
            .as_ref()
            .ok_or(RenderError::EmptyDocument)?;
        let page = document
            .get_page(page_index)
            .ok_or(RenderError::InvalidPage(page_index))?;
        let (width_points, height_points) = page.get_size();
        let scale = dpi / 72.0;
        let width = pixel_dimension(width_points, scale)?;
        let height = pixel_dimension(height_points, scale)?;

        let surface = ImageSurface::create(Format::ARgb32, width, height)?;
        let context = Context::new(&surface)?;
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint()?;
        context.scale(scale, scale);
        page.render(&context);
        surface.flush();

        Ok(surface)
    }
}

fn pixel_dimension(points: f64, scale: f64) -> Result<i32, RenderError> {
    let pixels = (points * scale).ceil();
    if !pixels.is_finite() || pixels <= 0.0 || pixels > i32::MAX as f64 {
        return Err(RenderError::InvalidDimensions);
    }
    Ok(pixels as i32)
}

fn page_orientation(width: f64, height: f64) -> Orientation {
    if width > height {
        Orientation::Landscape
    } else {
        Orientation::Portrait
    }
}

fn extract_page_contents(page: &PopplerPage) -> Vec<GraphicsElement> {
    page.get_text()
        .filter(|text| !text.is_empty())
        .map(|text| {
            vec![GraphicsElement::Text {
                content: text.to_owned(),
                font: "sans-serif".to_owned(),
                size: 12.0,
                x: 0.0,
                y: 0.0,
                color: Color::black(),
            }]
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_page_orientation_from_dimensions() {
        assert_eq!(page_orientation(842.0, 595.0), Orientation::Landscape);
        assert_eq!(page_orientation(595.0, 842.0), Orientation::Portrait);
        assert_eq!(page_orientation(500.0, 500.0), Orientation::Portrait);
    }

    #[test]
    fn validates_pixel_dimensions() {
        assert_eq!(pixel_dimension(72.0, 2.0).unwrap(), 144);
        assert!(matches!(
            pixel_dimension(0.0, 1.0),
            Err(RenderError::InvalidDimensions)
        ));
    }
}
