//! Masonry PDF-page canvas and its Xilem view adapter.

use xilem::core::{MessageContext, Mut, ViewMarker};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NoAction, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::kurbo::{Affine, Size};
use xilem::masonry::peniko::{Color, Fill, ImageBrush, ImageFormat};
use xilem::masonry::properties::ObjectFit;
use xilem::masonry::vello::peniko::{ImageAlphaType, ImageData};
use xilem::masonry::vello::Scene;
use xilem::{MessageResult, Pod, View, ViewCtx};

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

/// Masonry leaf widget that paints one rendered PDF page.
pub struct PdfCanvasWidget {
    image: Option<CanvasImage>,
    zoom: f64,
}

impl PdfCanvasWidget {
    fn new(image: Option<CanvasImage>, zoom: f64) -> Self {
        Self { image, zoom }
    }

    fn update(this: &mut WidgetMut<'_, Self>, image: Option<CanvasImage>, zoom: f64) {
        this.widget.image = image;
        this.widget.zoom = zoom;
        this.ctx.request_layout();
    }
}

impl Widget for PdfCanvasWidget {
    type Action = NoAction;

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        let natural = self
            .image
            .as_ref()
            .map_or(Size::new(595.0, 842.0), CanvasImage::size);
        bc.constrain(Size::new(
            natural.width * self.zoom,
            natural.height * self.zoom,
        ))
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

        if let Some(image) = &self.image {
            let transform = ObjectFit::Contain.affine_to_fill(ctx.size(), image.size());
            scene.draw_image(&image.brush, transform);
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
    image: Option<CanvasImage>,
    zoom: f64,
}

/// Create a PDF canvas view.
pub fn pdf_canvas(image: Option<CanvasImage>, zoom: f64) -> PdfCanvas {
    PdfCanvas { image, zoom }
}

impl ViewMarker for PdfCanvas {}

impl<State, Action> View<State, Action, ViewCtx> for PdfCanvas {
    type Element = Pod<PdfCanvasWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        (
            ctx.create_pod(PdfCanvasWidget::new(self.image.clone(), self.zoom)),
            (),
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        (): &mut Self::ViewState,
        _: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _: &mut State,
    ) {
        if self.image != prev.image || self.zoom != prev.zoom {
            PdfCanvasWidget::update(&mut element, self.image.clone(), self.zoom);
        }
    }

    fn teardown(&self, (): &mut Self::ViewState, ctx: &mut ViewCtx, element: Mut<'_, Self::Element>) {
        ctx.teardown_leaf(element);
    }

    fn message(
        &self,
        (): &mut Self::ViewState,
        message: &mut MessageContext,
        _: Mut<'_, Self::Element>,
        _: &mut State,
    ) -> MessageResult<Action> {
        tracing::error!(?message, "unexpected message delivered to PdfCanvas");
        MessageResult::Stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
