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
pub use job::{PrintJob, JobStatus};
pub use document::{Document, Page, PageSize, Orientation};
pub use printer::{PrinterInfo, PrinterCapabilities};
pub use constants::*;

// Re-export commonly used types
pub use uuid::Uuid;
