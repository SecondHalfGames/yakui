/// A button on the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// The user's primary mouse button. This is usually the left mouse button.
    One,

    /// The user's secondary mouse button. This is usually the right mouse
    /// button.
    Two,

    /// The user's third mouse button. This is usually the middle mouse button.
    Three,
}

/// A physical key on the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum KeyboardKey {
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    Return,
    Left,
    Right,
    Home,
    End,
    Up,
    Down,
    PageUp,
    PageDown,
}

/// The state of the keyboard modifier keys
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifiersState {
    /// Whether the SHIFT key is now down.
    pub shift: bool,
    /// Whether the CTRL key is now down.
    pub ctrl: bool,
    /// Whether the ALT key is now down.
    pub alt: bool,
    /// Whether the logo key is now down. This is the "windows" key on PC and
    /// "command" key on Mac.
    pub logo: bool,
}
