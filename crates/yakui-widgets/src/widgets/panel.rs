use std::cell::RefCell;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::colors;
use crate::util::widget_children;

const _RESIZE_HANDLE_WIDTH: f32 = 6.0;

/// Incomplete widget: represents a resizable panel on the sides, top, or bottom
/// of an area. Currently blocked on figuring out the best way to detect input
/// for resizing: add a child widget for this purpose, or handle the hit
/// detection and movement within a single widget?
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Panel {
    pub kind: PanelKind,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PanelKind {
    Side,
    TopBottom,
}

impl Panel {
    pub fn side() -> Self {
        Self {
            kind: PanelKind::Side,
        }
    }

    pub fn top_bottom() -> Self {
        Self {
            kind: PanelKind::TopBottom,
        }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<PanelResponse> {
        widget_children::<PanelWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct PanelWidget {
    props: Panel,
    size: RefCell<Vec2>,
}

pub type PanelResponse = ();

impl Widget for PanelWidget {
    type Props<'a> = Panel;
    type Response = PanelResponse;

    fn new() -> Self {
        Self {
            props: Panel::side(),
            size: RefCell::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = input.constrain(*self.size.borrow());

        match self.props.kind {
            PanelKind::Side => {
                if input.max.y.is_finite() {
                    size.y = input.max.y;
                }
            }

            PanelKind::TopBottom => {
                if input.max.x.is_finite() {
                    size.x = input.max.x;
                }
            }
        }

        let child_constraints = Constraints::tight(size);

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, child_constraints);
            size = size.max(child_size);
        }

        // TODO: If our children overflowed the size set in the panel, we should
        // recompute the layout of our children. If any of our children depend
        // on our size, their layout will change next frame.

        *self.size.borrow_mut() = size;

        input.constrain(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();
        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = colors::BACKGROUND_2;
        rect.add(ctx.paint);

        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseMoved(Some(_pos)) => {
                // TODO: How do we know where the mouse is relative to our
                // widget? We don't have access to the LayoutDom here.
                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
