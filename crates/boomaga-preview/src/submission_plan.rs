//! Pure planning for deterministic downstream CUPS submissions.
use boomaga_core::{DuplexMode, PageRange, PrintOptions};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionJob {
    pub copies: u32,
    pub pages: PageRange,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionPlan {
    pub jobs: Vec<SubmissionJob>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionPlanError {
    EmptyDocument,
    ZeroCopies,
    InvalidPageSelection,
}

impl fmt::Display for SubmissionPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => f.write_str("cannot print an empty document"),
            Self::ZeroCopies => f.write_str("copies must be greater than zero"),
            Self::InvalidPageSelection => {
                f.write_str("page selection exceeds the document page count")
            }
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
        let selected = match &options.page_range {
            Some(selection) => selection
                .pages(page_count)
                .map_err(|_| SubmissionPlanError::InvalidPageSelection)?,
            None => (1..=page_count).collect(),
        };
        let complete = PageRange::from_pages(&selected);
        let jobs = if options.collate {
            (0..options.copies)
                .map(|_| SubmissionJob {
                    copies: 1,
                    pages: complete.clone(),
                })
                .collect()
        } else if options.duplex == DuplexMode::None {
            vec![SubmissionJob {
                copies: options.copies,
                pages: complete,
            }]
        } else {
            let capacity = usize::from(options.pages_per_sheet as u8) * 2;
            selected
                .chunks(capacity)
                .map(|pages| SubmissionJob {
                    copies: options.copies,
                    pages: PageRange::from_pages(pages),
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
    use std::str::FromStr;

    fn job(copies: u32, pages: &str) -> SubmissionJob {
        SubmissionJob {
            copies,
            pages: PageRange::from_str(pages).unwrap(),
        }
    }
    #[test]
    fn collated_copies_repeat_the_complete_selection() {
        let o = PrintOptions {
            copies: 3,
            collate: true,
            page_range: Some(PageRange::from_str("1-3,7,9").unwrap()),
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(10, &o).unwrap().jobs,
            vec![job(1, "1-3,7,9"), job(1, "1-3,7,9"), job(1, "1-3,7,9")]
        );
    }
    #[test]
    fn uncollated_simplex_is_one_multi_copy_job() {
        let o = PrintOptions {
            copies: 3,
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(5, &o).unwrap().jobs,
            vec![job(3, "1-5")]
        );
    }
    #[test]
    fn uncollated_duplex_packs_selected_pages_into_physical_sheets() {
        let o = PrintOptions {
            copies: 3,
            duplex: DuplexMode::LongEdge,
            page_range: Some(PageRange::from_str("1-3,7,9").unwrap()),
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(10, &o).unwrap().jobs,
            vec![job(3, "1-2"), job(3, "3,7"), job(3, "9")]
        );
    }
    #[test]
    fn duplex_n_up_uses_selected_page_count_for_sheet_capacity() {
        let o = PrintOptions {
            copies: 2,
            duplex: DuplexMode::ShortEdge,
            pages_per_sheet: PagesPerSheet::Four,
            page_range: Some(PageRange::from_str("1-3,7,9,12-13").unwrap()),
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(13, &o).unwrap().jobs,
            vec![job(2, "1-3,7,9,12-13")]
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
        let o = PrintOptions {
            page_range: Some(PageRange::from_str("1,6").unwrap()),
            ..PrintOptions::default()
        };
        assert_eq!(
            SubmissionPlan::new(5, &o),
            Err(SubmissionPlanError::InvalidPageSelection)
        );
    }
}
