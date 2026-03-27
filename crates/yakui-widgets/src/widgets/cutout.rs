use crate::auto_builders;
use crate::shapes::RoundedRectangle;
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::util::{widget, widget_children};

/**
Displays an image based on the widget's screen-space position. Useful for
implementing a blur-behind effect.

Responds with [CutOutResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct CutOut {
    pub image: Option<TextureId>,
    pub image_color: Color,
    pub overlay_color: Color,
    pub min_size: Vec2,
    pub radius: f32,
}

auto_builders!(CutOut {
    image_color: Color,
    overlay_color: Color,
    min_size: Vec2,
    radius: f32,
});

impl CutOut {
    pub fn new<I>(image: I, overlay_color: Color) -> Self
    where
        I: Into<TextureId>,
    {
        Self {
            image: Some(image.into()),
            image_color: Color::WHITE,
            overlay_color,
            min_size: Vec2::ZERO,
            radius: 0.0,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<CutOutResponse> {
        widget::<CutOutWidget>(self)
    }

    #[track_caller]
    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<CutOutResponse> {
        widget_children::<CutOutWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct CutOutWidget {
    props: CutOut,
}

pub type CutOutResponse = ();

impl Widget for CutOutWidget {
    type Props<'a> = CutOut;
    type Response = CutOutResponse;

    fn new() -> Self {
        Self {
            props: CutOut {
                image: None,
                image_color: Color::WHITE,
                overlay_color: Color::CLEAR,
                min_size: Vec2::ZERO,
                radius: 0.0,
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
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

            RoundedRectangle::new(layout_node.rect, self.props.radius)
                .color(self.props.image_color)
                .texture((image, texture_rect))
                .add(ctx.paint);
        }

        RoundedRectangle::new(layout_node.rect, self.props.radius)
            .color(self.props.overlay_color)
            .add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
