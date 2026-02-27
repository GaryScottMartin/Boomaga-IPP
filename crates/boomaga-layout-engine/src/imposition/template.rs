//! Page layout templates for N-up and booklet layouts

use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use boomaga_core::{PageSize, Error, Result};
use crate::n_up::{PagePosition, PageResult};

/// Layout template for N-up and booklet layouts
#[derive(Debug, Clone)]
pub struct LayoutTemplate {
    /// Number of pages per sheet
    pages_per_sheet: u8,
    /// Output page size
    output_size: PageSize,
    /// Scaled page size
    scaled_size: (f64, f64),
    /// Page positions
    positions: Vec<PagePosition>,
}

impl LayoutTemplate {
    /// Create a new layout template
    pub fn new(
        pages_per_sheet: u8,
        output_size: PageSize,
        scaled_size: (f64, f64),
    ) -> Self {
        info!("Creating layout template: {} pages per sheet", pages_per_sheet);

        Self {
            pages_per_sheet,
            output_size,
            scaled_size,
            positions: Vec::new(),
        }
    }

    /// Generate page positions based on pages per sheet
    pub fn generate_positions(&mut self) -> Vec<PagePosition> {
        match self.pages_per_sheet {
            1 => vec![PagePosition::MiddleCenter],
            2 => vec![
                PagePosition::TopLeft,
                PagePosition::BottomRight,
            ],
            4 => vec![
                PagePosition::TopLeft,
                PagePosition::TopRight,
                PagePosition::BottomLeft,
                PagePosition::BottomRight,
            ],
            6 => vec![
                PagePosition::TopLeft,
                PagePosition::TopCenter,
                PagePosition::TopRight,
                PagePosition::BottomLeft,
                PagePosition::BottomCenter,
                PagePosition::BottomRight,
            ],
            8 => vec![
                PagePosition::TopLeft,
                PagePosition::TopCenter,
                PagePosition::TopRight,
                PagePosition::MiddleLeft,
                PagePosition::MiddleCenter,
                PagePosition::MiddleRight,
                PagePosition::BottomLeft,
                PagePosition::BottomRight,
            ],
            _ => vec![PagePosition::MiddleCenter],
        }
    }

    /// Generate pages for the template
    pub fn generate_pages(&self, input_pages: &[usize]) -> Vec<Vec<usize>> {
        let mut pages = Vec::new();
        let mut input_iter = input_pages.iter().cloned();

        for _ in 0..self.pages_per_sheet {
            let mut page = Vec::new();

            for _ in 0.. {
                if let Some(page_num) = input_iter.next() {
                    page.push(page_num);
                    if page.len() == (input_pages.len() as f64 / self.pages_per_sheet as f64) as usize
                        || page.len() * self.pages_per_sheet as usize >= input_pages.len() {
                        break;
                    }
                } else {
                    break;
                }
            }

            pages.push(page);
        }

        pages
    }

    /// Get position for a specific page
    pub fn get_position(&self, index: usize) -> Result<PagePosition> {
        let positions = self.generate_positions();

        if index >= positions.len() {
            return Err(Error::NotFound(format!(
                "Position index {} out of range (max: {})",
                index,
                positions.len()
            )));
        }

        Ok(positions[index])
    }

    /// Get scaled size
    pub fn scaled_size(&self) -> (f64, f64) {
        self.scaled_size
    }

    /// Get output size
    pub fn output_size(&self) -> PageSize {
        self.output_size
    }

    /// Get pages per sheet
    pub fn pages_per_sheet(&self) -> u8 {
        self.pages_per_sheet
    }
}

/// Preset layout templates
pub struct PresetLayout {
    pub name: &'static str,
    pub pages_per_sheet: u8,
    pub output_size: PageSize,
    pub description: &'static str,
}

impl PresetLayout {
    /// Get standard preset layouts
    pub fn presets() -> Vec<Self> {
        vec![
            PresetLayout {
                name: "Single Page",
                pages_per_sheet: 1,
                output_size: PageSize::A4,
                description: "One page per sheet",
            },
            PresetLayout {
                name: "2-Up",
                pages_per_sheet: 2,
                output_size: PageSize::A4,
                description: "Two pages per sheet",
            },
            PresetLayout {
                name: "4-Up",
                pages_per_sheet: 4,
                output_size: PageSize::A4,
                description: "Four pages per sheet",
            },
            PresetLayout {
                name: "8-Up",
                pages_per_sheet: 8,
                output_size: PageSize::A4,
                description: "Eight pages per sheet",
            },
        ]
    }

    /// Find preset by name
    pub fn find(name: &str) -> Option<Self> {
        Self::presets().into_iter().find(|p| p.name == name)
    }
}
