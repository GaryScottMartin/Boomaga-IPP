//! Booklet page layout algorithms

use boomaga_core::{PageSize, Error, Result};
use crate::n_up::{NUpCalculator, PagePosition, NUpLayout};
use tracing::{info, debug};

/// Booklet layout result
pub struct BookletLayout {
    /// The output pages
    pub pages: Vec<PageResult>,
    /// The output page size
    pub output_size: PageSize,
    /// Number of output pages
    pub page_count: usize,
    /// Booklet type
    pub booklet_type: BookletType,
    /// Page arrangement
    pub arrangement: PageArrangement,
}

/// Booklet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookletType {
    /// Standard booklet (saddle-stitched)
    Standard,
    /// Bypass stapled booklet
    BypassStapled,
    /// Double-sided saddle-stitched
    DoubleSidedSaddleStitched,
}

/// Page arrangement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageArrangement {
    /// Pages in correct order
    CorrectOrder,
    /// Pages reversed
    Reversed,
}

/// Booklet layout calculator
pub struct BookletCalculator {
    /// Booklet type
    booklet_type: BookletType,
    /// Margins
    margins: MarginConfig,
    /// Number of pages
    page_count: usize,
}

impl BookletCalculator {
    /// Create a new booklet calculator
    pub fn new(booklet_type: BookletType, page_count: usize) -> Result<Self> {
        if page_count == 0 {
            return Err(Error::Validation("Page count must be greater than 0".into()));
        }

        Ok(Self {
            booklet_type,
            margins: MarginConfig::default(),
            page_count,
        })
    }

    /// Calculate booklet layout
    pub fn calculate(&self, output_size: PageSize) -> Result<BookletLayout> {
        info!("Calculating {}-page booklet layout ({} pages per sheet)", self.page_count, self.page_count);

        // For a booklet, we need an even number of pages
        if self.page_count % 2 != 0 {
            return Err(Error::Validation("Booklet requires an even number of pages".into()));
        }

        // Calculate number of output sheets needed
        let output_sheets = (self.page_count + 3) / 4; // Ceiling division

        info!("Need {} output sheets for {} input pages", output_sheets, self.page_count);

        // For a booklet, we need 4 pages per sheet at minimum
        let pages_per_sheet = std::cmp::max(4, (self.page_count + 3) / 4);

        let mut pages = Vec::new();

        // Generate pages in booklet order
        for sheet_index in 0..output_sheets {
            // Determine input pages for this sheet
            let input_pages = self.generate_sheet_pages(sheet_index);

            // Create output page for this sheet
            let output_page = self.create_booklet_page(sheet_index, input_pages, output_size)?;

            pages.push(output_page);
        }

        Ok(BookletLayout {
            pages,
            output_size,
            page_count: pages.len(),
            booklet_type: self.booklet_type,
            arrangement: PageArrangement::CorrectOrder,
        })
    }

    /// Generate input pages for a sheet
    fn generate_sheet_pages(&self, sheet_index: usize) -> Vec<usize> {
        let mut pages = Vec::new();

        // Calculate which input pages belong to this sheet
        for i in 0..4 {
            let page_num = sheet_index * 4 + i;

            // Skip empty pages at the end
            if page_num < self.page_count {
                pages.push(page_num + 1); // Convert to 1-based
            }
        }

        pages
    }

    /// Create a booklet page
    fn create_booklet_page(
        &self,
        sheet_index: usize,
        input_pages: Vec<usize>,
        output_size: PageSize,
    ) -> Result<PageResult> {
        let position = self.determine_page_position(sheet_index, input_pages.clone())?;

        let content = None; // In production, would render the page content

        Ok(PageResult {
            output_page: sheet_index + 1,
            input_pages,
            position,
            content,
        })
    }

    /// Determine page position based on sheet index
    fn determine_page_position(
        &self,
        sheet_index: usize,
        input_pages: Vec<usize>,
    ) -> Result<PagePosition> {
        // For booklet, we need to arrange pages correctly
        // The first sheet has pages 4, 3, 2, 1
        // The second sheet has pages 8, 7, 6, 5
        // And so on...

        let page_order = match input_pages.as_slice() {
            [1, 2, 3, 4] if sheet_index == 0 => vec![4, 3, 2, 1],
            [5, 6, 7, 8] if sheet_index == 1 => vec![8, 7, 6, 5],
            _ => input_pages,
        };

        // Determine position based on input page numbers
        match page_order.as_slice() {
            [4, 1, 2, 3] => Ok(PagePosition::TopLeft), // Right side, left side, etc.
            [1, 2, 3, 4] => Ok(PagePosition::BottomRight),
            _ => {
                // Fallback to standard position
                Ok(PagePosition::TopLeft)
            }
        }
    }

    /// Check if a page count is suitable for booklet printing
    pub fn is_suitable(&self) -> bool {
        // Booklets work best with even numbers of pages
        self.page_count % 2 == 0
    }

    /// Get minimum page count for booklet
    pub fn min_page_count() -> usize {
        4 // Minimum 4 pages for a booklet
    }
}

/// Margin configuration
#[derive(Debug, Clone, Copy)]
pub struct MarginConfig {
    pub margin: f64,
    pub gutter: f64,
    pub crop_marks: bool,
    pub bleed_marks: bool,
}

impl Default for MarginConfig {
    fn default() -> Self {
        Self {
            margin: 0.0,
            gutter: 0.0,
            crop_marks: false,
            bleed_marks: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booklet_calculation() {
        let calculator = BookletCalculator::new(BookletType::Standard, 8).unwrap();

        let output_size = PageSize::A4;
        let result = calculator.calculate(output_size).unwrap();

        assert_eq!(result.page_count, 2); // 8 pages need 2 sheets
        assert_eq!(result.booklet_type, BookletType::Standard);
    }

    #[test]
    fn test_odd_page_count() {
        let calculator = BookletCalculator::new(BookletType::Standard, 7);

        assert!(calculator.is_err());
    }
}
