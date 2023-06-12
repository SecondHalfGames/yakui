use yakui_core::widget::{PaintContext, Widget};
use yakui_core::Response;

use crate::ignore_debug::IgnoreDebug;
use crate::util::widget;

type DrawCallback = Box<dyn Fn(PaintContext<'_>) + 'static>;

/**
Allows the user to draw arbitrary graphics in a region.

Responds with [CanvasResponse].
*/
#[derive(Debug)]
pub struct Canvas {
    draw: IgnoreDebug<Option<DrawCallback>>,
}

impl Canvas {
    pub fn new(draw: impl Fn(PaintContext<'_>) + 'static) -> Self {
        Self {
            draw: IgnoreDebug(Some(Box::new(draw))),
        }
    }

    pub fn show(self) -> Response<CanvasWidget> {
        widget::<CanvasWidget>(self)
    }
}

#[derive(Debug)]
pub struct CanvasWidget {
    props: Canvas,
}

pub type CanvasResponse = ();

impl Widget for CanvasWidget {
    type Props = Canvas;
    type Response = CanvasResponse;

    fn new() -> Self {
        Self {
            props: Canvas {
                draw: IgnoreDebug(None),
            },
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        if let IgnoreDebug(Some(draw)) = &self.props.draw {
            (draw)(ctx);
        }
    }
}
