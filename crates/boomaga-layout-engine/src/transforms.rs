//! Page transformation operations

use boomaga_core::{PageSize, Orientation, Error, Result};
use tracing::{debug, info};

/// Page transformation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformOperation {
    /// Rotate 90 degrees clockwise
    Rotate90,
    /// Rotate 180 degrees
    Rotate180,
    /// Rotate 90 degrees counter-clockwise
    Rotate270,
    /// Flip horizontally
    FlipHorizontal,
    /// Flip vertically
    FlipVertical,
    /// Scale to fit
    ScaleFit { width: f64, height: f64 },
    /// Custom transformation
    Custom { x: f64, y: f64, rotation: f64, scale: f64 },
}

/// Transformed page result
#[derive(Debug, Clone)]
pub struct TransformedPage {
    /// Original page
    pub original_page: usize,
    /// Transform applied
    pub transform: TransformOperation,
    /// Transformed size
    pub transformed_size: (f64, f64),
    /// Page position
    pub position: (f64, f64),
}

/// Page transformer
pub struct PageTransformer {
    /// Default rotation
    default_rotation: f64,
    /// Default scale
    default_scale: f64,
}

impl Default for PageTransformer {
    fn default() -> Self {
        Self {
            default_rotation: 0.0,
            default_scale: 1.0,
        }
    }
}

impl PageTransformer {
    /// Create a new page transformer
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform a page with default settings
    pub fn transform(&self, page_size: PageSize, orientation: Orientation) -> TransformedPage {
        self.transform_with_rotation(page_size, orientation, self.default_rotation)
    }

    /// Transform a page with specific rotation
    pub fn transform_with_rotation(
        &self,
        page_size: PageSize,
        orientation: Orientation,
        rotation_degrees: f64,
    ) -> TransformedPage {
        let radians = rotation_degrees.to_radians();

        let (width, height) = if orientation.is_landscape() {
            // Swap dimensions for landscape
            (page_size.height_points(), page_size.width_points())
        } else {
            (page_size.width_points(), page_size.height_points())
        };

        // Apply rotation
        let (transformed_width, transformed_height) = if rotation_degrees % 180.0 == 90.0 {
            (height * self.default_scale, width * self.default_scale)
        } else {
            (width * self.default_scale, height * self.default_scale)
        };

        TransformedPage {
            original_page: 0, // TODO: Track original page
            transform: TransformOperation::Rotate90, // Simplified
            transformed_size: (transformed_width, transformed_height),
            position: (0.0, 0.0),
        }
    }

    /// Transform a page with custom settings
    pub fn transform_custom(
        &self,
        page_size: PageSize,
        x: f64,
        y: f64,
        rotation: f64,
        scale: f64,
    ) -> TransformedPage {
        let (width, height) = (page_size.width_points() * scale, page_size.height_points() * scale);

        TransformedPage {
            original_page: 0,
            transform: TransformOperation::Custom { x, y, rotation, scale },
            transformed_size: (width, height),
            position: (x, y),
        }
    }

    /// Apply page rotation
    pub fn rotate(&self, degrees: f64) -> TransformedPage {
        // In production, this would be called on existing pages
        TransformedPage {
            original_page: 0,
            transform: TransformOperation::Rotate90,
            transformed_size: (0.0, 0.0),
            position: (0.0, 0.0),
        }
    }

    /// Apply horizontal flip
    pub fn flip_horizontal(&self) -> TransformedPage {
        TransformedPage {
            original_page: 0,
            transform: TransformOperation::FlipHorizontal,
            transformed_size: (0.0, 0.0),
            position: (0.0, 0.0),
        }
    }

    /// Apply vertical flip
    pub fn flip_vertical(&self) -> TransformedPage {
        TransformedPage {
            original_page: 0,
            transform: TransformOperation::FlipVertical,
            transformed_size: (0.0, 0.0),
            position: (0.0, 0.0),
        }
    }

    /// Calculate page position within output sheet
    pub fn calculate_position(
        &self,
        index: usize,
        total_pages: usize,
        output_size: PageSize,
        pages_per_sheet: u8,
    ) -> (f64, f64) {
        // Simple grid-based positioning
        let margin = 20.0;
        let page_width = (output_size.width_points() - margin * 2.0) / (pages_per_sheet as f64);
        let page_height = (output_size.height_points() - margin * 2.0) / (total_pages as f64).max(1.0);

        let row = (index as f64) / (pages_per_sheet as f64) as f64;
        let col = index as f64 % (pages_per_sheet as f64);

        (
            margin + col * page_width,
            margin + row * page_height,
        )
    }
}

/// Page rotation calculator
pub struct PageRotationCalculator {
    /// Target orientation
    target_orientation: Orientation,
    /// Current orientation
    current_orientation: Orientation,
}

impl PageRotationCalculator {
    /// Create a new rotation calculator
    pub fn new(target_orientation: Orientation) -> Self {
        Self {
            target_orientation,
            current_orientation: Orientation::Portrait,
        }
    }

    /// Get required rotation (in degrees)
    pub fn required_rotation(&self) -> f64 {
        match (self.current_orientation, self.target_orientation) {
            (Orientation::Portrait, Orientation::Portrait) => 0.0,
            (Orientation::Portrait, Orientation::Landscape) => 90.0,
            (Orientation::Portrait, Orientation::UpsideDownPortrait) => 180.0,
            (Orientation::Portrait, Orientation::UpsideDownLandscape) => 270.0,

            (Orientation::Landscape, Orientation::Portrait) => 270.0,
            (Orientation::Landscape, Orientation::Landscape) => 0.0,
            (Orientation::Landscape, Orientation::UpsideDownPortrait) => 90.0,
            (Orientation::Landscape, Orientation::UpsideDownLandscape) => 180.0,

            (Orientation::UpsideDownPortrait, Orientation::Portrait) => 180.0,
            (Orientation::UpsideDownPortrait, Orientation::Landscape) => 270.0,
            (Orientation::UpsideDownPortrait, Orientation::UpsideDownPortrait) => 0.0,
            (Orientation::UpsideDownPortrait, Orientation::UpsideDownLandscape) => 90.0,

            (Orientation::UpsideDownLandscape, Orientation::Portrait) => 90.0,
            (Orientation::UpsideDownLandscape, Orientation::Landscape) => 180.0,
            (Orientation::UpsideDownLandscape, Orientation::UpsideDownPortrait) => 270.0,
            (Orientation::UpsideDownLandscape, Orientation::UpsideDownLandscape) => 0.0,
        }
    }

    /// Set current orientation
    pub fn set_current(&mut self, orientation: Orientation) {
        self.current_orientation = orientation;
    }

    /// Calculate rotation for multiple pages
    pub fn calculate_rotations(&self, pages: &[usize]) -> Vec<f64> {
        pages.iter()
            .map(|_| self.required_rotation())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_rotation() {
        let calculator = PageRotationCalculator::new(Orientation::Landscape);
        calculator.set_current(Orientation::Portrait);

        assert_eq!(calculator.required_rotation(), 90.0);
    }

    #[test]
    fn test_transformed_size() {
        let transformer = PageTransformer::new();
        let page_size = PageSize::A4;

        let result = transformer.transform(page_size, Orientation::Portrait);
        assert_eq!(result.transformed_size.0, 595.0);
        assert_eq!(result.transformed_size.1, 842.0);
    }
}
