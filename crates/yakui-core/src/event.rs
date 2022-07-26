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
    MouseButtonChanged {
        /// Which mouse button was changed.
        button: MouseButton,

        /// Whether the button is now down.
        down: bool,
    },

    /// A key changed, telling whether it is now pressed.
    KeyChanged {
        /// Which key's state was changed.
        key: KeyboardKey,

        /// Whether the key is now down.
        down: bool,
    },

    /// The state of the keyboard modifiers keys changed.
    ModifiersChanged {
        /// Whether the SHIFT key is now down.
        shift: bool,
        /// Whether the CTRL key is now down.
        ctrl: bool,
        /// Whether the ALT key is now down.
        alt: bool,
        /// Whether the logo key is now down. This is the "windows" key on PC
        /// and "command" key on Mac.
        logo: bool,
    },

    /// A Unicode codepoint was typed in the window.
    TextInput(char),
}

/// An event that can be handled by an individual widget.
#[derive(Debug)]
#[non_exhaustive]
pub enum WidgetEvent {
    /// The mouse entered the widget's layout rectangle.
    MouseEnter,

    /// The mouse left the widget's layout rectangle.
    MouseLeave,

    /// The mouse moved.
    MouseMoved(Option<Vec2>),

    /// A mouse button changed state while the cursor was inside the widget's
    /// layout rectangle.
    #[non_exhaustive]
    MouseButtonChanged {
        /// Which button was changed.
        button: MouseButton,

        /// Whether the button is down or up.
        down: bool,

        /// Whether the button is inside the widget's layout rectangle.
        inside: bool,

        /// The position of the mouse cursor at the time of the event.
        position: Vec2,
    },

    /// A keyboard button changed.
    KeyChanged(KeyboardKey, bool),

    /// Text was sent to the widget.
    TextInput(char),
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
        const MOUSE_INSIDE = 1;

        /// Notify this widget of mouse events occuring outside its layout
        /// rectangle.
        const MOUSE_OUTSIDE = 2;

        /// Notify this widget whenever the mouse cursor moves.
        const MOUSE_MOVE = 4;

        /// This widget can be focused.
        const FOCUS = 8;

        /// If this widget is focused, it should receive keyboard events.
        const FOCUSED_KEYBOARD = 16;

        /// Notify this widget of all mouse events.
        const MOUSE_ALL = Self::MOUSE_INSIDE.bits | Self::MOUSE_OUTSIDE.bits | Self::MOUSE_MOVE.bits;
    }
}
