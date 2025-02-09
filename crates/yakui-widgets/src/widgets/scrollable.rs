use std::cell::Cell;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Scrollable {
    pub direction: Option<ScrollDirection>,
}

impl Scrollable {
    pub fn none() -> Self {
        Scrollable { direction: None }
    }

    pub fn vertical() -> Self {
        Scrollable {
            direction: Some(ScrollDirection::Y),
        }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<ScrollableResponse> {
        widget_children::<ScrollableWidget, F>(children, self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Y,
}

#[derive(Debug)]
pub struct ScrollableWidget {
    props: Scrollable,
    scroll_position: Cell<Vec2>,
    canvas_size: Cell<Vec2>,
}

pub type ScrollableResponse = ();

impl Widget for ScrollableWidget {
    type Props<'a> = Scrollable;
    type Response = ScrollableResponse;

    fn new() -> Self {
        Self {
            props: Scrollable::none(),
            scroll_position: Cell::new(Vec2::ZERO),
            canvas_size: Cell::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);

        let node = ctx.dom.get_current();
        let mut canvas_size = Vec2::ZERO;

        let child_constraints = match self.props.direction {
            None => constraints,
            Some(ScrollDirection::Y) => Constraints {
                min: Vec2::new(constraints.min.x, 0.0),
                max: Vec2::new(constraints.max.x, f32::INFINITY),
            },
        };

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, child_constraints);
            canvas_size = canvas_size.max(child_size);
        }
        self.canvas_size.set(canvas_size);

        let size = constraints.constrain(canvas_size);

        let max_scroll_position = (canvas_size - size).max(Vec2::ZERO);
        let mut scroll_position = self
            .scroll_position
            .get()
            .min(max_scroll_position)
            .max(Vec2::ZERO);

        match self.props.direction {
            None => scroll_position = Vec2::ZERO,
            Some(ScrollDirection::Y) => scroll_position.x = 0.0,
        }

        self.scroll_position.set(scroll_position);

        for &child in &node.children {
            ctx.layout.set_pos(child, -scroll_position);
        }

        size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.paint(child);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseScroll { delta } => {
                let pos = self.scroll_position.get();
                self.scroll_position.set(pos + delta);
                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
    }
}
