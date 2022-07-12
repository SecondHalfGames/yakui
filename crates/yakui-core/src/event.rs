use glam::Vec2;

use crate::geometry::Rect;
use crate::input::MouseButton;

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    SetViewport(Rect),
    MoveMouse(Option<Vec2>),
    MouseButtonChanged(MouseButton, bool),
}

#[allow(clippy::enum_variant_names)]
#[non_exhaustive]
pub enum WidgetEvent {
    MouseEnter,
    MouseLeave,
    MouseButtonChangedInside(MouseButton, bool),
    MouseButtonChangedOutside(MouseButton, bool),
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct EventInterest: u8 {
        const MOUSE_INSIDE  = 0b0000_0001;
        const MOUSE_OUTSIDE = 0b0000_0010;
    }
}
