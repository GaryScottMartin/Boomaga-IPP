//! Deterministic saddle-stitch booklet planning.

use boomaga_core::{Error, Result};

/// One imposed side of a physical booklet sheet.
///
/// Page indices are zero-based. `None` represents a blank introduced while
/// padding the source document to a multiple of four pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookletSide {
    pub sheet_index: usize,
    pub is_back: bool,
    pub slots: [Option<usize>; 2],
}

/// Complete booklet plan in printer output order: front, back, front, back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookletPlan {
    pub sides: Vec<BookletSide>,
    pub source_page_count: usize,
    pub padded_page_count: usize,
}

impl BookletPlan {
    pub fn new(page_count: usize) -> Result<Self> {
        if page_count == 0 {
            return Err(Error::Validation(
                "Booklet page count must be greater than 0".into(),
            ));
        }

        let padded_page_count = page_count.div_ceil(4) * 4;
        let sheet_count = padded_page_count / 4;
        let mut sides = Vec::with_capacity(sheet_count * 2);
        let source_slot = |page_index| (page_index < page_count).then_some(page_index);

        for sheet_index in 0..sheet_count {
            let outer_left = padded_page_count - sheet_index * 2 - 1;
            let inner_left = sheet_index * 2;
            let inner_right = inner_left + 1;
            let outer_right = outer_left - 1;

            sides.push(BookletSide {
                sheet_index,
                is_back: false,
                slots: [source_slot(outer_left), source_slot(inner_left)],
            });
            sides.push(BookletSide {
                sheet_index,
                is_back: true,
                slots: [source_slot(inner_right), source_slot(outer_right)],
            });
        }

        Ok(Self {
            sides,
            source_page_count: page_count,
            padded_page_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(page_count: usize) -> Vec<[Option<usize>; 2]> {
        BookletPlan::new(page_count)
            .unwrap()
            .sides
            .into_iter()
            .map(|side| side.slots)
            .collect()
    }

    #[test]
    fn eight_pages_are_ordered_for_two_saddle_stitched_sheets() {
        assert_eq!(
            slots(8),
            vec![
                [Some(7), Some(0)],
                [Some(1), Some(6)],
                [Some(5), Some(2)],
                [Some(3), Some(4)],
            ]
        );
    }

    #[test]
    fn incomplete_signatures_are_padded_with_blank_slots() {
        assert_eq!(
            slots(6),
            vec![
                [None, Some(0)],
                [Some(1), None],
                [Some(5), Some(2)],
                [Some(3), Some(4)],
            ]
        );
        assert_eq!(slots(1), vec![[None, Some(0)], [None, None]]);
    }

    #[test]
    fn empty_documents_are_rejected() {
        assert!(BookletPlan::new(0).is_err());
    }
}
