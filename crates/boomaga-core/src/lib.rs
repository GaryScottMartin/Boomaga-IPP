//! Core shared logic for boomaga virtual printer
//!
//! This crate provides common types, error handling, and utilities shared
//! across all boomaga components.

pub mod error;
pub mod job;
pub mod document;
pub mod printer;
pub mod constants;

pub use error::{Error, Result};
pub use job::{JobStatus, FileType, JobMetadata, JobPriority, PrintJobRequest, PrintOptions, DuplexMode, PagesPerSheet, MarginMode, PageInfo};
pub use document::{Document, Page, PageSize, Orientation, PageContents, GraphicsElement, PathElement, Color};
pub use printer::{PrinterInfo, PrinterCapabilities, PageLayout};
pub use constants::*;

// Re-export commonly used types
pub use uuid::Uuid;
