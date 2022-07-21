use yakui_core::dom::Dom;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::widget::Widget;
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
pub struct Panel {}

impl Panel {
    pub fn side() -> Self {
        Self {}
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<PanelWidget> {
        widget_children::<PanelWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct PanelWidget {
    props: Panel,
    size: Vec2,
}

pub type PanelResponse = ();

impl Widget for PanelWidget {
    type Props = Panel;
    type Response = PanelResponse;

    fn new() -> Self {
        Self {
            props: Panel::side(),
            size: Vec2::ZERO,
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();
        let mut size = self.size;

        let child_constraints = Constraints {
            min: self.size,
            max: input.max,
        };

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, child_constraints);
            size = size.max(child_size);
        }

        input.constrain(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let layout_node = layout.get(dom.current()).unwrap();
        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = colors::BACKGROUND_2;
        paint.add_rect(rect);

        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
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
