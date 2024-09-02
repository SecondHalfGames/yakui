use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::Vec2;
use yakui_core::input::MouseButton;
use yakui_core::widget::{EventContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Draggable {}

impl Draggable {
    pub fn new() -> Self {
        Draggable {}
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<DraggableResponse> {
        widget_children::<DraggableWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct DraggableWidget {
    current_drag: Option<DragState>,
}

#[derive(Debug)]
struct DragState {
    start_position: Vec2,
    offset_from_mouse: Vec2,
    mouse_position: Vec2,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct DraggableResponse {
    pub dragging: Option<Dragging>,
}

#[derive(Debug, Clone, Copy)]
pub struct Dragging {
    pub start: Vec2,
    pub current: Vec2,
}

impl Widget for DraggableWidget {
    type Props<'a> = Draggable;
    type Response = DraggableResponse;

    fn new() -> Self {
        Self { current_drag: None }
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {
        let dragging = self.current_drag.as_ref().map(|drag| Dragging {
            start: drag.start_position,
            current: drag.mouse_position + drag.offset_from_mouse,
        });

        DraggableResponse { dragging }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                position,
                ..
            } => {
                if down && inside {
                    let node = ctx.layout.get(ctx.dom.current()).unwrap();

                    self.current_drag = Some(DragState {
                        start_position: node.rect.pos(),
                        offset_from_mouse: node.rect.pos() - position,
                        mouse_position: position,
                    });

                    EventResponse::Sink
                } else if !down && self.current_drag.is_some() {
                    self.current_drag = None;
                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }
            WidgetEvent::MouseMoved(Some(position)) => {
                if let Some(drag) = &mut self.current_drag {
                    drag.mouse_position = position;
                }

                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
