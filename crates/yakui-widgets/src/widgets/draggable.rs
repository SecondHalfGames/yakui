use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::Vec2;
use yakui_core::input::MouseButton;
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::util::widget_children;

#[derive(Debug)]
#[non_exhaustive]
pub struct Draggable {}

impl Draggable {
    pub fn new() -> Self {
        Draggable {}
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<DraggableWidget> {
        widget_children::<DraggableWidget, F>(children, self)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct DraggableWidget {
    held: bool,
    down_position: Vec2,
    mouse_position: Vec2,
}

#[non_exhaustive]
pub struct DraggableResponse {
    pub dragging: Option<Dragging>,
}

#[derive(Debug)]
pub struct Dragging {
    pub start: Vec2,
    pub current: Vec2,
}

impl Widget for DraggableWidget {
    type Props = Draggable;
    type Response = DraggableResponse;

    fn new() -> Self {
        Self {
            held: false,
            down_position: Vec2::ZERO,
            mouse_position: Vec2::ZERO,
        }
    }

    fn update(&mut self, _props: Self::Props) -> Self::Response {
        let dragging = if self.held {
            Some(Dragging {
                start: self.down_position,
                current: self.mouse_position,
            })
        } else {
            None
        };

        DraggableResponse { dragging }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                position,
                ..
            } => {
                if down && inside {
                    self.held = true;
                    self.down_position = position;
                    self.mouse_position = position;
                    EventResponse::Sink
                } else if !down && self.held {
                    self.held = false;
                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }
            WidgetEvent::MouseMoved(Some(pos)) => {
                if self.held {
                    self.mouse_position = pos;
                }

                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
