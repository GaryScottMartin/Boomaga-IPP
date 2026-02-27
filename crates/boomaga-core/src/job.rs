//! Print job types and handling

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{Error, Result};

/// Unique identifier for a print job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for JobId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for JobId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<JobId> for Uuid {
    fn from(job_id: JobId) -> Self {
        job_id.0
    }
}

/// Status of a print job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued and waiting
    Queued,
    /// Job is being processed
    Processing,
    /// Job completed successfully
    Completed,
    /// Job was cancelled by user
    Cancelled,
    /// Job failed
    Failed,
    /// Job held for review
    Held,
    /// Job aborted
    Aborted,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "Queued"),
            JobStatus::Processing => write!(f, "Processing"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Cancelled => write!(f, "Cancelled"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Held => write!(f, "Held"),
            JobStatus::Aborted => write!(f, "Aborted"),
        }
    }
}

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Urgent priority
    Urgent = 3,
}

impl std::fmt::Display for JobPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobPriority::Low => write!(f, "Low"),
            JobPriority::Normal => write!(f, "Normal"),
            JobPriority::High => write!(f, "High"),
            JobPriority::Urgent => write!(f, "Urgent"),
        }
    }
}

/// Print job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub job_id: JobId,
    pub name: String,
    pub user: String,
    pub created_at: std::time::SystemTime,
    pub completed_at: Option<std::time::SystemTime>,
    pub pages_printed: usize,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub pages: Vec<PageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub page_number: usize,
    pub width: f64,
    pub height: f64,
    pub orientation: Orientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Pdf,
    PostScript,
    Ps,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Pdf => write!(f, "PDF"),
            FileType::PostScript => write!(f, "PostScript"),
            FileType::Ps => write!(f, "PostScript"),
        }
    }
}

/// Print job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJobRequest {
    pub job_id: JobId,
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub printer_name: Option<String>,
    pub options: PrintOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintOptions {
    pub copies: u32,
    pub collate: bool,
    pub duplex: DuplexMode,
    pub orientation: Orientation,
    pub page_range: Option<(usize, usize)>,
    pub pages_per_sheet: PagesPerSheet,
    pub scale: f64,
    pub margins: MarginMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DuplexMode {
    /// No duplex
    None,
    /// Long edge binding (standard book)
    LongEdge,
    /// Short edge binding (calendar style)
    ShortEdge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PagesPerSheet {
    /// One page per sheet
    One = 1,
    /// Two pages per sheet
    Two = 2,
    /// Four pages per sheet
    Four = 4,
    /// Six pages per sheet
    Six = 6,
    /// Eight pages per sheet
    Eight = 8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarginMode {
    /// No margins
    None,
    /// Minimum margins
    Minimum,
    /// Normal margins
    Normal,
    /// Wide margins
    Wide,
    /// Custom { margins }
    Custom { top: f64, bottom: f64, left: f64, right: f64 },
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            copies: 1,
            collate: false,
            duplex: DuplexMode::None,
            orientation: Orientation::Portrait,
            page_range: None,
            pages_per_sheet: PagesPerSheet::One,
            scale: 1.0,
            margins: MarginMode::Normal,
        }
    }
}

impl PrintOptions {
    /// Validate print options
    pub fn validate(&self) -> Result<()> {
        if self.copies == 0 {
            return Err(Error::Validation("Copies must be greater than 0".into()));
        }

        if !matches!(self.page_range, None | Some((_, _))) {
            // Range will be validated when pages are loaded
        }

        Ok(())
    }

    /// Check if this is a booklet job
    pub fn is_booklet(&self) -> bool {
        matches!(self.pages_per_sheet, PagesPerSheet::Two)
    }
}

/// Job completion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatistics {
    pub job_id: JobId,
    pub duration: std::time::Duration,
    pub pages_processed: usize,
    pub bytes_processed: u64,
    pub success_rate: f64,
    pub average_processing_time_per_page: std::time::Duration,
}
