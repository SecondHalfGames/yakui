use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{ManagedTextureId, Response};

use crate::util::widget;

/**
Displays an image.

Responds with [ImageResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Image {
    pub image: Option<ManagedTextureId>,
    pub size: Vec2,
}

impl Image {
    pub fn new(image: ManagedTextureId, size: Vec2) -> Self {
        Self {
            image: Some(image),
            size,
        }
    }

    pub fn show(self) -> Response<ImageWidget> {
        widget::<ImageWidget>(self)
    }
}

#[derive(Debug)]
pub struct ImageWidget {
    props: Image,
}

pub type ImageResponse = ();

impl Widget for ImageWidget {
    type Props = Image;
    type Response = ImageResponse;

    fn new() -> Self {
        Self {
            props: Image {
                image: None,
                size: Vec2::ZERO,
            },
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, _ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        input.constrain_min(self.props.size)
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        if let Some(image) = self.props.image {
            let mut rect = PaintRect::new(layout_node.rect);
            rect.color = Color::WHITE;
            rect.texture = Some((image, Rect::ONE));
            ctx.paint.add_rect(rect);
        }
    }
}
