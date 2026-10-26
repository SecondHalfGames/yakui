use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::widget::{EventContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
`Opaque` blocks all mouse events from proceeding further. It's intended to be
used as a top-level element in windows, panels, pop-ups, and similar widgets
that don't want mouse input to go through them.
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Opaque {}

impl Opaque {
    pub fn new() -> Self {
        Self {}
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<OpaqueResponse> {
        widget_children::<OpaqueWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct OpaqueWidget;

pub type OpaqueResponse = ();

impl Widget for OpaqueWidget {
    type Props = Opaque;
    type Response = OpaqueResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn event_interest(&self, _props: &Self::Props) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_MOVE
    }

    fn event(
        &mut self,
        _props: &Self::Props,
        _ctx: EventContext<'_>,
        event: &WidgetEvent,
    ) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter
            | WidgetEvent::MouseLeave
            | WidgetEvent::MouseButtonChanged { down: true, .. }
            | WidgetEvent::MouseScroll { .. } => EventResponse::Sink,
            _ => EventResponse::Bubble,
        }
    }
}
