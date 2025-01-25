//! Types and utilities for handling UI navigation with mice, keyboards, and
//! gamepads.

use crate::dom::Dom;
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::widget::NavigateContext;
use crate::WidgetId;

/// Possible directions that a user can navigate in when using a gamepad or
/// keyboard in a UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum NavDirection {
    /// The next widget in the layout, used when the user presses tab.
    Next,

    /// The previous widget in the layout, used if the user presses shift+tab.
    Previous,

    Down,
    Up,
    Left,
    Right,
}

pub(crate) fn navigate(
    dom: &Dom,
    layout: &LayoutDom,
    input: &InputState,
    dir: NavDirection,
) -> Option<WidgetId> {
    let mut current = input.selection();

    while let Some(id) = current {
        let node = dom.get(id).unwrap();
        let ctx = NavigateContext { dom, layout, input };

        if let Some(new_id) = ctx.try_navigate(id, dir) {
            return Some(new_id);
        }

        current = node.parent;
    }

    None
}
