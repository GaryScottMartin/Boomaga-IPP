//! Masonry PDF-page canvas and its Xilem view adapter.

use xilem::core::{MessageContext, MessageResult, Mut, View, ViewMarker};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::keyboard::{Key, NamedKey};
use xilem::masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, EventCtx, LayoutCtx, PaintCtx, PointerEvent,
    PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::kurbo::{Affine, Size};
use xilem::masonry::peniko::{Color, Fill, ImageBrush, ImageFormat};
use xilem::masonry::properties::ObjectFit;
use xilem::masonry::vello::peniko::{ImageAlphaType, ImageData};
use xilem::masonry::vello::Scene;
use xilem::{Pod, ViewCtx};

use crate::app::{AppData, PreviewShortcut};

/// A rendered PDF page ready for Masonry/Vello painting.
#[derive(Clone, PartialEq)]
pub struct CanvasImage {
    brush: ImageBrush,
    width: u32,
    height: u32,
}

impl CanvasImage {
    /// Build an image from Cairo `Format::ARgb32` bytes on little-endian Linux.
    ///
    /// Cairo stores those pixels as premultiplied BGRA bytes. Row padding must
    /// be removed by the caller before constructing this value.
    pub fn from_cairo_bgra(
        pixels: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<Self, CanvasImageError> {
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            return Err(CanvasImageError::InvalidBufferLength {
                expected,
                actual: pixels.len(),
            });
        }

        let image = ImageData {
            data: pixels.into(),
            format: ImageFormat::Bgra8,
            alpha_type: ImageAlphaType::AlphaPremultiplied,
            width,
            height,
        };

        Ok(Self {
            brush: ImageBrush::new(image),
            width,
            height,
        })
    }

    fn size(&self) -> Size {
        Size::new(self.width as f64, self.height as f64)
    }
}

/// Invalid rendered-page image data.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CanvasImageError {
    #[error("expected {expected} BGRA bytes, received {actual}")]
    InvalidBufferLength { expected: usize, actual: usize },
}

fn grid_dimensions(pages_per_sheet: u8) -> (usize, usize) {
    match pages_per_sheet {
        1 => (1, 1),
        2 => (2, 1),
        4 => (2, 2),
        6 => (3, 2),
        8 => (4, 2),
        _ => (1, 1),
    }
}

fn grid_slot(index: usize, pages_per_sheet: u8, vertical: bool) -> usize {
    if vertical && matches!(pages_per_sheet, 4 | 6 | 8) {
        let (columns, rows) = grid_dimensions(pages_per_sheet);
        (index % rows) * columns + index / rows
    } else {
        index
    }
}

fn imposed_sheet_size(source_size: Size, pages_per_sheet: u8) -> Size {
    if matches!(pages_per_sheet, 2 | 6 | 8) {
        Size::new(source_size.height, source_size.width)
    } else {
        source_size
    }
}

/// Masonry leaf widget that paints one rendered PDF page.
pub struct PdfCanvasWidget {
    images: Vec<Option<CanvasImage>>,
    pages_per_sheet: u8,
    vertical_fill: bool,
    zoom: f64,
}

impl PdfCanvasWidget {
    fn new(
        images: Vec<Option<CanvasImage>>,
        pages_per_sheet: u8,
        vertical_fill: bool,
        zoom: f64,
    ) -> Self {
        Self {
            images,
            pages_per_sheet,
            vertical_fill,
            zoom,
        }
    }

    fn update(
        this: &mut WidgetMut<'_, Self>,
        images: Vec<Option<CanvasImage>>,
        pages_per_sheet: u8,
        vertical_fill: bool,
        zoom: f64,
    ) {
        this.widget.images = images;
        this.widget.pages_per_sheet = pages_per_sheet;
        this.widget.vertical_fill = vertical_fill;
        this.widget.zoom = zoom;
        this.ctx.request_layout();
    }
}

impl Widget for PdfCanvasWidget {
    type Action = PreviewShortcut;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if matches!(event, PointerEvent::Down(_)) {
            ctx.request_focus();
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        let TextEvent::Keyboard(event) = event else {
            return;
        };
        if !event.state.is_down() || event.repeat {
            return;
        }

        let shortcut = match &event.key {
            Key::Character(key) if key == " " => Some(PreviewShortcut::NextPage),
            Key::Named(NamedKey::ArrowRight) => Some(PreviewShortcut::NextPage),
            Key::Character(key) if key.eq_ignore_ascii_case("n") => Some(PreviewShortcut::NextPage),
            Key::Named(NamedKey::ArrowLeft) => Some(PreviewShortcut::PreviousPage),
            Key::Character(key) if key.eq_ignore_ascii_case("p") => {
                Some(PreviewShortcut::PreviousPage)
            }
            Key::Character(key) if key == "+" || key == "=" => Some(PreviewShortcut::ZoomIn),
            Key::Character(key) if key == "-" => Some(PreviewShortcut::ZoomOut),
            Key::Character(key) if key == "0" => Some(PreviewShortcut::ResetZoom),
            _ => None,
        };
        if let Some(shortcut) = shortcut {
            ctx.submit_action::<Self::Action>(shortcut);
            ctx.set_handled();
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        let natural = self
            .images
            .iter()
            .flatten()
            .next()
            .map_or(Size::new(595.0, 842.0), CanvasImage::size);
        let sheet_size = imposed_sheet_size(natural, self.pages_per_sheet);
        let preferred = Size::new(sheet_size.width * self.zoom, sheet_size.height * self.zoom);
        bc.constrain_aspect_ratio(preferred.height / preferred.width, preferred.width)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        let bounds = ctx.size().to_rect();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(255, 255, 255),
            None,
            &bounds,
        );

        let (columns, rows) = grid_dimensions(self.pages_per_sheet);
        let cell = Size::new(
            ctx.size().width / columns as f64,
            ctx.size().height / rows as f64,
        );
        for (index, image) in self.images.iter().enumerate() {
            let Some(image) = image else { continue };
            let slot = grid_slot(index, self.pages_per_sheet, self.vertical_fill);
            let x = (slot % columns) as f64 * cell.width;
            let y = (slot / columns) as f64 * cell.height;
            let fit = ObjectFit::Contain.affine_to_fill(cell, image.size());
            scene.draw_image(&image.brush, Affine::translate((x, y)) * fit);
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.set_label("PDF page preview");
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> tracing::Span {
        tracing::trace_span!("PdfCanvasWidget", id = id.trace())
    }
}

/// Xilem view that owns the reactive inputs to [`PdfCanvasWidget`].
#[must_use = "View values do nothing unless provided to Xilem"]
pub struct PdfCanvas {
    images: Vec<Option<CanvasImage>>,
    pages_per_sheet: u8,
    vertical_fill: bool,
    zoom: f64,
}

/// Create a PDF canvas view.
pub fn pdf_canvas(
    images: Vec<Option<CanvasImage>>,
    pages_per_sheet: u8,
    vertical_fill: bool,
    zoom: f64,
) -> PdfCanvas {
    PdfCanvas {
        images,
        pages_per_sheet,
        vertical_fill,
        zoom,
    }
}

impl ViewMarker for PdfCanvas {}

impl View<AppData, (), ViewCtx> for PdfCanvas {
    type Element = Pod<PdfCanvasWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut AppData) -> (Self::Element, Self::ViewState) {
        (
            ctx.with_action_widget(|ctx| {
                ctx.create_pod(PdfCanvasWidget::new(
                    self.images.clone(),
                    self.pages_per_sheet,
                    self.vertical_fill,
                    self.zoom,
                ))
            }),
            (),
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        (): &mut Self::ViewState,
        _: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _: &mut AppData,
    ) {
        if self.images != prev.images
            || self.pages_per_sheet != prev.pages_per_sheet
            || self.vertical_fill != prev.vertical_fill
            || self.zoom != prev.zoom
        {
            PdfCanvasWidget::update(
                &mut element,
                self.images.clone(),
                self.pages_per_sheet,
                self.vertical_fill,
                self.zoom,
            );
        }
    }

    fn teardown(
        &self,
        (): &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.teardown_leaf(element);
    }

    fn message(
        &self,
        (): &mut Self::ViewState,
        message: &mut MessageContext,
        _: Mut<'_, Self::Element>,
        app_state: &mut AppData,
    ) -> MessageResult<()> {
        match message.take_message::<PreviewShortcut>() {
            Some(shortcut) => {
                app_state.apply_shortcut(*shortcut);
                MessageResult::Action(())
            }
            None => {
                tracing::error!(?message, "unexpected message delivered to PdfCanvas");
                MessageResult::Stale
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xilem::masonry::core::NewWidget;
    use xilem::masonry::testing::TestHarness;
    use xilem::masonry::theme::default_property_set;

    #[test]
    fn focused_canvas_emits_shortcuts_from_real_keyboard_events() {
        let widget = NewWidget::new(PdfCanvasWidget::new(Vec::new(), 1, false, 1.0));
        let mut harness = TestHarness::create(default_property_set(), widget);
        let canvas_id = harness.root_id();

        harness.mouse_click_on(canvas_id);
        assert_eq!(harness.focused_widget_id(), Some(canvas_id));

        harness.process_text_event(TextEvent::key_down(Key::Named(NamedKey::ArrowRight)));
        assert_eq!(
            harness.pop_action::<PreviewShortcut>(),
            Some((PreviewShortcut::NextPage, canvas_id))
        );
    }

    #[test]
    fn rejects_incorrect_pixel_buffer_length() {
        let error = match CanvasImage::from_cairo_bgra(vec![0; 7], 2, 1) {
            Ok(_) => panic!("invalid buffer length was accepted"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            CanvasImageError::InvalidBufferLength {
                expected: 8,
                actual: 7,
            }
        );
    }

    #[test]
    fn accepts_exact_pixel_buffer_length() {
        let image = CanvasImage::from_cairo_bgra(vec![255; 8], 2, 1).unwrap();

        assert_eq!(image.size(), Size::new(2.0, 1.0));
    }

    #[test]
    fn uses_expected_n_up_grids() {
        assert_eq!(grid_dimensions(1), (1, 1));
        assert_eq!(grid_dimensions(2), (2, 1));
        assert_eq!(grid_dimensions(4), (2, 2));
        assert_eq!(grid_dimensions(6), (3, 2));
        assert_eq!(grid_dimensions(8), (4, 2));
    }

    #[test]
    fn two_up_uses_landscape_sheet_orientation() {
        let portrait = Size::new(595.0, 842.0);

        assert_eq!(imposed_sheet_size(portrait, 1), portrait);
        assert_eq!(imposed_sheet_size(portrait, 2), Size::new(842.0, 595.0));
        assert_eq!(imposed_sheet_size(portrait, 4), portrait);
        assert_eq!(imposed_sheet_size(portrait, 6), Size::new(842.0, 595.0));
        assert_eq!(imposed_sheet_size(portrait, 8), Size::new(842.0, 595.0));
    }

    #[test]
    fn vertical_fill_matches_classic_boomaga_order() {
        let four_up: Vec<_> = (0..4).map(|index| grid_slot(index, 4, true)).collect();
        let eight_up: Vec<_> = (0..8).map(|index| grid_slot(index, 8, true)).collect();

        assert_eq!(four_up, vec![0, 2, 1, 3]);
        let six_up: Vec<_> = (0..6).map(|index| grid_slot(index, 6, true)).collect();
        assert_eq!(six_up, vec![0, 3, 1, 4, 2, 5]);
        assert_eq!(eight_up, vec![0, 4, 1, 5, 2, 6, 3, 7]);
    }

    #[test]
    fn horizontal_fill_is_row_major() {
        let slots: Vec<_> = (0..8).map(|index| grid_slot(index, 8, false)).collect();
        assert_eq!(slots, (0..8).collect::<Vec<_>>());
    }
}
