//! Navigation event handlers for Xilem

use crate::app_xilem::AppData;

/// Navigate to previous page
pub fn navigate_prev(data: &mut AppData) {
    data.previous_page();
}

/// Navigate to next page
pub fn navigate_next(data: &mut AppData) {
    data.next_page();
}

/// Navigate to first page
pub fn navigate_first(data: &mut AppData) {
    data.first_page();
}

/// Navigate to last page
pub fn navigate_last(data: &mut AppData) {
    data.last_page();
}

/// Navigate to specific page
pub fn navigate_to_page(data: &mut AppData, page: usize) {
    if let Some(document) = &data.document {
        if page < document.page_count() {
            data.current_page = page;
        }
    }
}
