//! Window management for Xilem

use xilem::WindowConfig;
use crate::app_xilem::AppData;

/// Create the main window configuration
pub fn create_window() -> WindowConfig<AppData> {
    WindowConfig::new()
        .title("Boomaga Preview")
        .size((1200, 800))
        .build()
}
