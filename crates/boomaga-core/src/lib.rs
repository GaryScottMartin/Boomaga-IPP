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
pub use job::{JobStatus, JobMetadata, JobPriority, PrintJobRequest, PrintOptions, PageInfo, JobId};
pub use document::{Document, Page, PageSize, Orientation, PageContents, GraphicsElement, PathElement, Color, FileType, PagesPerSheet, MarginMode, DuplexMode};
pub use printer::{PrinterInfo, PrinterCapabilities, PageLayout};

// Re-export constants explicitly
pub use constants::{
    APP_NAME, APP_VERSION, APP_DESCRIPTION,
    CONFIG_DIR, CACHE_DIR, STATE_DIR,
    DEFAULT_IPC_SOCKET, DEFAULT_DBUS_SERVICE,
    DEFAULT_DBUS_PATH, DEFAULT_IPP_PORT,
    DEFAULT_THUMBNAIL_SIZE, DEFAULT_PREVIEW_ZOOM_LEVELS,
    DEFAULT_MAX_JOB_HISTORY, DEFAULT_TIMEOUT_SECS,
    DEFAULT_MAX_CONCURRENT_JOBS, DEFAULT_WORKER_THREADS,
    DEFAULT_JOB_QUEUE_SIZE, AppConfig,
    IPC_SOCKET_PATH, DBUS_SERVICE_NAME, MAX_CONCURRENT_JOBS, WORKER_THREADS, JOB_QUEUE_SIZE,
};

// Re-export commonly used types
pub use uuid::Uuid;
