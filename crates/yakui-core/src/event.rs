//! Defines the events that can be sent to yakui and handled by widgets.

use glam::Vec2;

use crate::geometry::Rect;
use crate::input::{KeyboardKey, MouseButton};

/// An event that can be handled by yakui.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// The viewport has changed. This can mean resizing as well as positioning.
    ViewportChanged(Rect),

    /// The mouse cursor moved. If `None`, indicates that the mouse moved
    /// outside the window.
    CursorMoved(Option<Vec2>),

    /// A mouse button changed, telling whether it is now pressed.
    MouseButtonChanged(MouseButton, bool),

    /// A key changed, telling whether it is now pressed.
    KeyChanged(KeyboardKey, bool),

    /// A Unicode codepoint was typed in the window.
    TextInput(char),
}

/// An event that can be handled by an individual widget.
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
#[non_exhaustive]
pub enum WidgetEvent {
    /// The mouse entered the widget's layout rectangle.
    MouseEnter,

    /// The mouse left the widget's layout rectangle.
    MouseLeave,

    /// A mouse button changed state while the cursor was inside the widget's
    /// layout rectangle.
    MouseButtonChanged(MouseButton, bool),

    /// A mouse button changed state while the cursor was outside the widget's
    /// rectangle.
    MouseButtonChangedOutside(MouseButton, bool),

    /// The widget was focused or unfocused.
    FocusChanged(bool),
}

/// Responses that can be given to an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
    /// Bubble the event. This gives other widgets or the application the chance
    /// to process the event.
    Bubble,

    /// Sink the event. This stops the event from propagating and tells the host
    /// application that it should not consider the event.
    Sink,
}

bitflags::bitflags! {
    /// A bitfield of events that a widget can register to be notified about.
    #[derive(Default)]
    pub struct EventInterest: u8 {
        /// Notify this widget of mouse events occuring within its layout
        /// rectangle.
        const MOUSE_INSIDE  = 0b0000_0001;

        /// Notify this widget of mouse events occuring outside its layout
        /// rectangle.
        const MOUSE_OUTSIDE = 0b0000_0010;

        /// Notify this widget of all mouse events.
        const MOUSE = Self::MOUSE_INSIDE.bits | Self::MOUSE_OUTSIDE.bits;

        /// This widget can be focused.
        const FOCUS = 0b0000_0100;
    }
}
