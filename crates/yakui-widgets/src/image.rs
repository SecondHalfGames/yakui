use yakui_core::dom::Dom;
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::{Color3, Constraints, Index, Rect, Vec2, Widget};

use crate::util::widget;

#[derive(Debug, Clone)]
pub struct Image {
    pub image: Index,
    pub size: Vec2,
}

#[derive(Debug)]
pub struct ImageWidget {
    props: Image,
}

pub type ImageResponse = ();

impl Widget for ImageWidget {
    type Props = Image;
    type Response = ImageResponse;

    fn new(_index: Index, props: Self::Props) -> Self {
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
        let viewport = layout.viewport();
        let size = layout_node.rect.size() / viewport.size();
        let pos = (layout_node.rect.pos() + viewport.pos()) / viewport.size();

        let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
        rect.color = Color3::WHITE;
        rect.texture = Some((self.props.image, Rect::ONE));
        output.add_rect(rect);
    }

    fn respond(&mut self) -> Self::Response {}
}

pub fn image(image: Index, size: Vec2) -> ImageResponse {
    widget::<ImageWidget>(Image { image, size })
}
