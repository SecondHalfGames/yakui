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
