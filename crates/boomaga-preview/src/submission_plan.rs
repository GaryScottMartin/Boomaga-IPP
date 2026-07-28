//! Pure planning for deterministic downstream CUPS submissions.
use boomaga_core::{DuplexMode, PrintOptions};
use std::{fmt, ops::RangeInclusive};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionJob {
    pub copies: u32,
    pub page_range: RangeInclusive<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionPlan {
    pub jobs: Vec<SubmissionJob>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionPlanError {
    EmptyDocument,
    ZeroCopies,
    InvalidPageRange {
        first: usize,
        last: usize,
        page_count: usize,
    },
}

impl fmt::Display for SubmissionPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => f.write_str("cannot print an empty document"),
            Self::ZeroCopies => f.write_str("copies must be greater than zero"),
            Self::InvalidPageRange {
                first,
                last,
                page_count,
            } => write!(
                f,
                "page range {first}-{last} is invalid for a {page_count}-page document"
            ),
        }
    }
}
impl std::error::Error for SubmissionPlanError {}

impl SubmissionPlan {
    pub fn new(page_count: usize, options: &PrintOptions) -> Result<Self, SubmissionPlanError> {
        if page_count == 0 {
            return Err(SubmissionPlanError::EmptyDocument);
        }
        if options.copies == 0 {
            return Err(SubmissionPlanError::ZeroCopies);
        }
        let (first, last) = options.page_range.unwrap_or((1, page_count));
        if first == 0 || first > last || last > page_count {
            return Err(SubmissionPlanError::InvalidPageRange {
                first,
                last,
                page_count,
            });
        }
        let range = first..=last;
        let jobs = if options.collate {
            (0..options.copies)
                .map(|_| SubmissionJob {
                    copies: 1,
                    page_range: range.clone(),
                })
                .collect()
        } else if options.duplex == DuplexMode::None {
            vec![SubmissionJob {
                copies: options.copies,
                page_range: range,
            }]
        } else {
            let pages_per_physical_sheet = usize::from(options.pages_per_sheet as u8) * 2;
            (first..=last)
                .step_by(pages_per_physical_sheet)
                .map(|start| SubmissionJob {
                    copies: options.copies,
                    page_range: start..=last.min(start + pages_per_physical_sheet - 1),
                })
                .collect()
        };
        Ok(Self { jobs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boomaga_core::PagesPerSheet;
    fn job(copies: u32, first: usize, last: usize) -> SubmissionJob {
        SubmissionJob {
            copies,
            page_range: first..=last,
        }
    }
    #[test]
    fn collated_copies_repeat_the_complete_range() {
        let o = PrintOptions {
            copies: 3,
            collate: true,
            page_range: Some((2, 7)),
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(10, &o).unwrap().jobs,
            vec![job(1, 2, 7), job(1, 2, 7), job(1, 2, 7)]
        );
    }
    #[test]
    fn uncollated_simplex_is_one_multi_copy_job() {
        let o = PrintOptions {
            copies: 3,
            ..PrintOptions::default()
        };
        assert_eq!(SubmissionPlan::new(5, &o).unwrap().jobs, vec![job(3, 1, 5)]);
    }
    #[test]
    fn uncollated_duplex_batches_physical_sheets_and_odd_tail() {
        let o = PrintOptions {
            copies: 3,
            duplex: DuplexMode::LongEdge,
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(5, &o).unwrap().jobs,
            vec![job(3, 1, 2), job(3, 3, 4), job(3, 5, 5)]
        );
    }
    #[test]
    fn duplex_n_up_and_range_set_sheet_width() {
        let o = PrintOptions {
            copies: 2,
            duplex: DuplexMode::ShortEdge,
            pages_per_sheet: PagesPerSheet::Four,
            page_range: Some((3, 18)),
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(20, &o).unwrap().jobs,
            vec![job(2, 3, 10), job(2, 11, 18)]
        );
    }
    #[test]
    fn invalid_inputs_are_rejected() {
        assert_eq!(
            SubmissionPlan::new(0, &PrintOptions::default()),
            Err(SubmissionPlanError::EmptyDocument)
        );
        let zero = PrintOptions {
            copies: 0,
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(1, &zero),
            Err(SubmissionPlanError::ZeroCopies)
        );
        for range in [(0, 1), (3, 2), (1, 6)] {
            let o = PrintOptions {
                page_range: Some(range),
                ..PrintOptions::default()
            };
            assert_eq!(
                SubmissionPlan::new(5, &o),
                Err(SubmissionPlanError::InvalidPageRange {
                    first: range.0,
                    last: range.1,
                    page_count: 5
                })
            );
        }
    }
}
