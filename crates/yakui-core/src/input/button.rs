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
    Tab,
    Space,
    Backspace,
    Delete,
    Return,
}
