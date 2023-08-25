use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::util::{widget, widget_children};

/**
Displays an image based on the widget's screen-space position. Useful for
implementing a blur-behind effect.

Responds with [CutOutResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CutOut {
    pub image: Option<TextureId>,
    pub overlay_color: Color,
    pub min_size: Vec2,
}

impl CutOut {
    pub fn new<I>(image: I, overlay_color: Color) -> Self
    where
        I: Into<TextureId>,
    {
        Self {
            image: Some(image.into()),
            overlay_color,
            min_size: Vec2::ZERO,
        }
    }

    pub fn show(self) -> Response<CutOutWidget> {
        widget::<CutOutWidget>(self)
    }

    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<CutOutWidget> {
        widget_children::<CutOutWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct CutOutWidget {
    props: CutOut,
}

pub type CutOutResponse = ();

impl Widget for CutOutWidget {
    type Props = CutOut;
    type Response = CutOutResponse;

    fn new() -> Self {
        Self {
            props: CutOut {
                image: None,
                overlay_color: Color::CLEAR,
                min_size: Vec2::ZERO,
            },
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = self.props.min_size;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        if let Some(image) = self.props.image {
            let texture_rect = Rect::from_pos_size(
                (layout_node.rect.pos() - ctx.layout.viewport().pos())
                    / ctx.layout.viewport().size(),
                layout_node.rect.size() / ctx.layout.viewport().size(),
            );

            let mut rect = PaintRect::new(layout_node.rect);
            rect.color = Color::WHITE;
            rect.texture = Some((image, texture_rect));
            rect.add(ctx.paint);
        }

        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = self.props.overlay_color;
        rect.add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
