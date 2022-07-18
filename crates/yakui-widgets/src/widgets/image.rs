use yakui_core::dom::Dom;
use yakui_core::geometry::{Color3, Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::widget::Widget;
use yakui_core::{Response, TextureId};

use crate::util::widget;

/**
Displays an image.

Responds with [ImageResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Image {
    pub image: Option<TextureId>,
    pub size: Vec2,
}

impl Image {
    pub fn new(image: TextureId, size: Vec2) -> Self {
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

    fn layout(&self, _dom: &Dom, _layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        input.constrain(self.props.size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, output: &mut PaintDom) {
        let layout_node = layout.get(dom.current()).unwrap();

        if let Some(image) = self.props.image {
            let mut rect = PaintRect::new(layout_node.rect);
            rect.color = Color3::WHITE;
            rect.texture = Some((image, Rect::ONE));
            output.add_rect(rect);
        }
    }
}
