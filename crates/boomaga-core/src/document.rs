//! Document types and handling

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a document (PDF or PostScript)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub title: String,
    pub author: Option<String>,
    pub creator: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub pages: Vec<Page>,
}

impl Document {
    /// Create a new document
    pub fn new(
        id: String,
        file_path: PathBuf,
        file_type: FileType,
    ) -> Self {
        Self {
            id,
            file_path,
            file_type,
            title: String::new(),
            author: None,
            creator: None,
            subject: None,
            keywords: Vec::new(),
            pages: Vec::new(),
        }
    }

    /// Add a page to the document
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
    }

    /// Get page count
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Get the last page
    pub fn last_page(&self) -> Option<&Page> {
        self.pages.last()
    }

    /// Check if document is empty
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Parse metadata from file
    pub async fn parse_metadata(&mut self) -> Result<()> {
        // TODO: Implement metadata parsing
        Ok(())
    }
}

/// Represents a single page in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub number: usize,
    pub width: f64,     // Points (1/72 inch)
    pub height: f64,    // Points (1/72 inch)
    pub orientation: Orientation,
    pub contents: PageContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageContents {
    /// Page contains vector graphics
    Vector(Vec<GraphicsElement>),
    /// Page contains bitmap/raster graphics
    Raster { width: usize, height: usize, data: Vec<u8> },
    /// Page contains raw PDF bytes
    Pdf { stream: Vec<u8> },
}

impl Page {
    /// Create a new blank page
    pub fn new(number: usize, width: f64, height: f64, orientation: Orientation) -> Self {
        Self {
            number,
            width,
            height,
            orientation,
            contents: PageContents::Vector(Vec::new()),
        }
    }

    /// Check if page has content
    pub fn has_content(&self) -> bool {
        match &self.contents {
            PageContents::Vector(elements) => !elements.is_empty(),
            PageContents::Raster { .. } => true,
            PageContents::Pdf { .. } => true,
        }
    }
}

/// Graphics element types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphicsElement {
    /// Path drawing (lines, curves)
    Path {
        elements: Vec<PathElement>,
        stroke: Option<Color>,
        fill: Option<Color>,
        stroke_width: f64,
    },
    /// Rectangle
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: f64,
    },
    /// Text element
    Text {
        content: String,
        font: String,
        size: f64,
        x: f64,
        y: f64,
        color: Color,
    },
    /// Image element
    Image {
        path: PathBuf,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

/// Path element types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathElement {
    MoveTo { x: f64, y: f64 },
    LineTo { x: f64, y: f64 },
    CurveTo { cp1: (f64, f64), cp2: (f64, f64), end: (f64, f64) },
    Close,
}

/// Page size types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageSize {
    /// Letter (8.5 x 11 inches)
    Letter,
    /// Legal (8.5 x 14 inches)
    Legal,
    /// A4 (210 x 297 mm)
    A4,
    /// A3 (297 x 420 mm)
    A3,
    /// A5 (148 x 210 mm)
    A5,
    /// B5 (176 x 250 mm)
    B5,
    /// Custom { width, height }
    Custom { width: f64, height: f64 },
}

impl PageSize {
    /// Get width in points
    pub fn width_points(&self) -> f64 {
        match self {
            PageSize::Letter => 612.0,   // 8.5 * 72
            PageSize::Legal => 612.0,    // 8.5 * 72
            PageSize::A4 => 595.0,       // 210 * 72 / 25.4
            PageSize::A3 => 842.0,       // 297 * 72 / 25.4
            PageSize::A5 => 420.0,       // 148 * 72 / 25.4
            PageSize::B5 => 498.0,       // 176 * 72 / 25.4
            PageSize::Custom { width, height } => *width,
        }
    }

    /// Get height in points
    pub fn height_points(&self) -> f64 {
        match self {
            PageSize::Letter => 792.0,   // 11 * 72
            PageSize::Legal => 1008.0,   // 14 * 72
            PageSize::A4 => 842.0,       // 297 * 72 / 25.4
            PageSize::A3 => 1191.0,      // 420 * 72 / 25.4
            PageSize::A5 => 595.0,       // 210 * 72 / 25.4
            PageSize::B5 => 709.0,       // 250 * 72 / 25.4
            PageSize::Custom { width, height } => *height,
        }
    }

    /// Get width in millimeters
    pub fn width_mm(&self) -> f64 {
        self.width_points() * 25.4 / 72.0
    }

    /// Get height in millimeters
    pub fn height_mm(&self) -> f64 {
        self.height_points() * 25.4 / 72.0
    }

    /// Create from millimeters
    pub fn from_mm(width_mm: f64, height_mm: f64) -> Self {
        PageSize::Custom {
            width: width_mm * 72.0 / 25.4,
            height: height_mm * 72.0 / 25.4,
        }
    }

    /// Get standard name
    pub fn as_str(&self) -> &'static str {
        match self {
            PageSize::Letter => "Letter",
            PageSize::Legal => "Legal",
            PageSize::A4 => "A4",
            PageSize::A3 => "A3",
            PageSize::A5 => "A5",
            PageSize::B5 => "B5",
            PageSize::Custom { .. } => "Custom",
        }
    }

    /// Get common standard sizes
    pub fn standard_sizes() -> Vec<Self> {
        vec![
            PageSize::A4,
            PageSize::A3,
            PageSize::A5,
            PageSize::Letter,
            PageSize::Legal,
            PageSize::B5,
        ]
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize::A4
    }
}

/// Page orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    /// Portrait (vertical)
    Portrait,
    /// Landscape (horizontal)
    Landscape,
    /// Upside-down portrait
    UpsideDownPortrait,
    /// Upside-down landscape
    UpsideDownLandscape,
}

impl Orientation {
    /// Rotate 90 degrees clockwise
    pub fn rotate_90(&self) -> Self {
        match self {
            Orientation::Portrait => Orientation::Landscape,
            Orientation::Landscape => Orientation::UpsideDownPortrait,
            Orientation::UpsideDownPortrait => Orientation::UpsideDownLandscape,
            Orientation::UpsideDownLandscape => Orientation::Portrait,
        }
    }

    /// Check if width is greater than height
    pub fn is_landscape(&self) -> bool {
        matches!(self, Orientation::Landscape | Orientation::UpsideDownLandscape)
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Portrait
    }
}

/// Color representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create from RGB
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create from RGBA
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Black
    pub fn black() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }

    /// White
    pub fn white() -> Self {
        Self { r: 255, g: 255, b: 255, a: 255 }
    }

    /// Red
    pub fn red() -> Self {
        Self { r: 255, g: 0, b: 0, a: 255 }
    }

    /// Green
    pub fn green() -> Self {
        Self { r: 0, g: 255, b: 0, a: 255 }
    }

    /// Blue
    pub fn blue() -> Self {
        Self { r: 0, g: 0, b: 255, a: 255 }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::black()
    }
}
