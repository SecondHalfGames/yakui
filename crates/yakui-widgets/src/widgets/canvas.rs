use yakui_core::widget::{PaintContext, Widget};
use yakui_core::Response;

use crate::ignore_debug::IgnoreDebug;
use crate::util::{widget, widget_children};

type DrawCallback = Box<dyn Fn(&mut PaintContext<'_>) + 'static>;

/**
Allows the user to draw arbitrary graphics in a region.

Responds with [CanvasResponse].
*/
#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Canvas {
    draw: IgnoreDebug<Option<DrawCallback>>,
}

impl Canvas {
    pub fn new(draw: impl Fn(&mut PaintContext<'_>) + 'static) -> Self {
        Self {
            draw: IgnoreDebug(Some(Box::new(draw))),
        }
    }

    pub fn show(self) -> Response<CanvasResponse> {
        widget::<CanvasWidget>(self)
    }

    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<CanvasResponse> {
        widget_children::<CanvasWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct CanvasWidget {
    props: Canvas,
}

pub type CanvasResponse = ();

impl Widget for CanvasWidget {
    type Props<'a> = Canvas;
    type Response = CanvasResponse;

    fn new() -> Self {
        Self {
            props: Canvas {
                draw: IgnoreDebug(None),
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        if let IgnoreDebug(Some(draw)) = &self.props.draw {
            (draw)(&mut ctx);
        }

        self.default_paint(ctx);
    }
}
