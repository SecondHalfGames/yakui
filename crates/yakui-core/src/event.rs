use glam::Vec2;

use crate::geometry::Rect;
use crate::input::MouseButton;

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    SetViewport(Rect),
    MoveMouse(Vec2),
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
