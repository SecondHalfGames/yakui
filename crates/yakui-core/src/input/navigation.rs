/// Possible directions that a user can navigate in when using a gamepad or
/// keyboard in a UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum NavDirection {
    Down,
    Up,
    Left,
    Right,

    /// The next widget in the layout, used when the user presses tab.
    Next,

    /// The previous widget in the layout, used if the user presses shift+tab.
    Previous,
}
