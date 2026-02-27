//! Print dialog widget

use druid::{
    widget::{Flex, Label, Slider, Selection,
    Split, TextField, UnitPoint},
    Color, Env, Widget,
};

/// Print dialog state
#[derive(Clone, Data, Lens)]
pub struct PrintDialog {
    /// Show the dialog
    pub show: bool,
    /// Document name
    pub document_name: String,
    /// Number of copies
    pub copies: u32,
    /// Duplex mode
    pub duplex: DuplexMode,
    /// Pages per sheet
    pub pages_per_sheet: PagesPerSheet,
    /// Page orientation
    pub orientation: Orientation,
    /// Print margins
    pub margins: MarginMode,
    /// Print button enabled
    pub print_enabled: bool,
}

/// Print modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum DuplexMode {
    None,
    LongEdge,
    ShortEdge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum PagesPerSheet {
    One = 1,
    Two = 2,
    Four = 4,
    Six = 6,
    Eight = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum MarginMode {
    None,
    Minimum,
    Normal,
    Wide,
}

impl Default for PrintDialog {
    fn default() -> Self {
        Self {
            show: false,
            document_name: String::new(),
            copies: 1,
            duplex: DuplexMode::None,
            pages_per_sheet: PagesPerSheet::One,
            orientation: Orientation::Portrait,
            margins: MarginMode::Normal,
            print_enabled: false,
        }
    }
}

impl PrintDialog {
    /// Create a new print dialog
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state based on document
    pub fn update_for_document(&mut self, document_name: String) {
        self.document_name = document_name;
        self.print_enabled = !document_name.is_empty();
    }

    /// Apply settings
    pub fn apply(&self) {
        tracing::info!("Print settings applied:");
        tracing::info!("  Copies: {}", self.copies);
        tracing::info!("  Duplex: {:?}", self.duplex);
        tracing::info!("  Pages per sheet: {:?}", self.pages_per_sheet);
        tracing::info!("  Orientation: {:?}", self.orientation);
        tracing::info!("  Margins: {:?}", self.margins);
    }

    /// Cancel dialog
    pub fn cancel(&mut self) {
        self.show = false;
    }
}

/// Print dialog widget
pub struct PrintDialogWidget {
    /// Print dialog state
    dialog: PrintDialog,
}

impl PrintDialogWidget {
    /// Create a new print dialog widget
    pub fn new() -> Self {
        Self {
            dialog: PrintDialog::new(),
        }
    }

    /// Set document
    pub fn set_document(&mut self, name: String) {
        self.dialog.update_for_document(name);
    }

    /// Show the dialog
    pub fn show_dialog(&mut self) {
        self.dialog.show = true;
    }

    /// Hide the dialog
    pub fn hide_dialog(&mut self) {
        self.dialog.show = false;
    }
}

impl Default for PrintDialogWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget<BoomagaApp> for PrintDialogWidget {
    fn event(&mut self, _ctx: &mut druid::EventCtx, _event: &druid::Event, data: &mut BoomagaApp, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &BoomagaApp, _env: &Env) {}

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &BoomagaApp, _data: &BoomagaApp, _env: &Env) {}

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, _data: &BoomagaApp, _env: &Env) -> druid::Size {
        ctx.constraints().constrain(bc)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &BoomagaApp, _env: &Env) {
        let size = ctx.size();

        // Draw dialog background
        ctx.fill(
            Rect::ZERO.with_size(size),
            &Color::WHITE,
        );

        // Draw dialog border
        ctx.stroke_line(
            (size.width / 2.0, 0.0),
            (size.width / 2.0, size.height),
            &Color::GRAY300,
        );
    }
}
