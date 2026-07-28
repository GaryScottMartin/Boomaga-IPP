//! Print job types and handling

use crate::document::{DuplexMode, MarginMode, Orientation, PagesPerSheet};
use crate::{Error, FileType, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

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
    pub file_path: std::path::PathBuf,
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

/// Print job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJobRequest {
    pub job_id: JobId,
    pub file_path: std::path::PathBuf,
    pub file_type: FileType,
    pub printer_name: Option<String>,
    pub options: PrintOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRange {
    ranges: Vec<(usize, usize)>,
}

impl PageRange {
    pub fn pages(&self, page_count: usize) -> Result<Vec<usize>> {
        let mut pages = Vec::new();
        for &(first, last) in &self.ranges {
            if last > page_count {
                return Err(Error::Validation(format!(
                    "page range {first}-{last} exceeds document page count {page_count}"
                )));
            }
            pages.extend(first..=last);
        }
        Ok(pages)
    }

    pub fn from_pages(pages: &[usize]) -> Self {
        let mut ranges = Vec::new();
        for &page in pages {
            match ranges.last_mut() {
                Some((_, last)) if page == *last + 1 => *last = page,
                _ => ranges.push((page, page)),
            }
        }
        Self { ranges }
    }
}

impl FromStr for PageRange {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut ranges = Vec::new();
        for item in input.split(',') {
            let item = item.trim();
            if item.is_empty() {
                return Err(Error::Validation(
                    "page selection contains an empty item".into(),
                ));
            }
            let mut bounds = item.split('-').map(str::trim);
            let first =
                bounds.next().unwrap().parse::<usize>().map_err(|_| {
                    Error::Validation(format!("invalid page selection item: {item}"))
                })?;
            let last = match bounds.next() {
                Some(value) => value.parse::<usize>().map_err(|_| {
                    Error::Validation(format!("invalid page selection item: {item}"))
                })?,
                None => first,
            };
            if bounds.next().is_some() || first == 0 || first > last {
                return Err(Error::Validation(format!(
                    "invalid page selection item: {item}"
                )));
            }
            if ranges
                .last()
                .is_some_and(|&(_, previous_last)| first <= previous_last)
            {
                return Err(Error::Validation(
                    "page selection must be ordered without overlaps".into(),
                ));
            }
            ranges.push((first, last));
        }
        if ranges.is_empty() {
            return Err(Error::Validation("page selection is empty".into()));
        }
        Ok(Self { ranges })
    }
}

impl std::fmt::Display for PageRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (first, last)) in self.ranges.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            if first == last {
                write!(f, "{first}")?;
            } else {
                write!(f, "{first}-{last}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintOptions {
    pub copies: u32,
    pub collate: bool,
    pub duplex: DuplexMode,
    pub orientation: Orientation,
    pub page_range: Option<PageRange>,
    pub pages_per_sheet: PagesPerSheet,
    pub scale: f64,
    pub margins: MarginMode,
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

        Ok(())
    }

    /// Check if this is a booklet job
    pub fn is_booklet(&self) -> bool {
        matches!(self.pages_per_sheet, PagesPerSheet::Two)
    }
}

#[cfg(test)]
mod page_range_tests {
    use super::PageRange;
    use std::str::FromStr;

    #[test]
    fn parses_and_formats_individual_pages_and_ranges() {
        let selection = PageRange::from_str("1-3, 7,9, 12-13").unwrap();
        assert_eq!(selection.to_string(), "1-3,7,9,12-13");
        assert_eq!(selection.pages(13).unwrap(), vec![1, 2, 3, 7, 9, 12, 13]);
    }

    #[test]
    fn rejects_malformed_unordered_overlapping_and_out_of_bounds_selections() {
        for input in ["", "0", "3-1", "1,,3", "1-2-3", "3,2", "1-3,3-4"] {
            assert!(PageRange::from_str(input).is_err(), "accepted {input:?}");
        }
        assert!(PageRange::from_str("1-3,7").unwrap().pages(6).is_err());
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
