use yakui_core::dom::Dom;
use yakui_core::geometry::{Color3, Constraints, Rect, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::widget::Widget;
use yakui_core::{Response, TextureId};

use crate::util::widget;

#[derive(Debug, Clone)]
pub struct Image {
    pub image: TextureId,
    pub size: Vec2,
}

impl Image {
    pub fn new(image: TextureId, size: Vec2) -> Self {
        Self { image, size }
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

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn layout(&self, _dom: &Dom, _layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        input.constrain(self.props.size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, output: &mut PaintDom) {
        let layout_node = layout.get(dom.current()).unwrap();

        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = Color3::WHITE;
        rect.texture = Some((self.props.image, Rect::ONE));
        output.add_rect(rect);
    }

    fn respond(&mut self) -> Self::Response {}
}
