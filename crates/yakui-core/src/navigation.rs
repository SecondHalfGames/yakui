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

impl NavDirection {
    /// Returns the corresponding [`NavDirections`] flag.
    pub const fn flag(self) -> NavDirections {
        match self {
            NavDirection::Next => NavDirections::NEXT,
            NavDirection::Previous => NavDirections::PREVIOUS,
            NavDirection::Down => NavDirections::DOWN,
            NavDirection::Up => NavDirections::UP,
            NavDirection::Left => NavDirections::LEFT,
            NavDirection::Right => NavDirections::RIGHT,
        }
    }
}

pub(crate) fn navigate(
    dom: &Dom,
    layout: &LayoutDom,
    input: &InputState,
    dir: NavDirection,
) -> Option<WidgetId> {
    let ctx = NavigateContext { dom, layout, input };
    let mut current = input.focus();

    // allow sequential navigation from the root if nothing is focused
    if current.is_none() && matches!(dir, NavDirection::Next | NavDirection::Previous) {
        return ctx.try_navigate(dom.root(), dir);
    }

    while let Some(id) = current {
        let node = dom.get(id)?;

        if let Some(new_id) = ctx.try_navigate(id, dir) {
            return Some(new_id);
        }

        current = node.parent;
    }

    None
}

bitflags::bitflags! {
    /// A bitfield of navigation directions.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    pub struct NavDirections: u8 {
        /// Next
        const NEXT = 1 << 0;
        /// Previous
        const PREVIOUS = 1 << 1;
        /// Down
        const DOWN = 1 << 2;
        /// Up
        const UP = 1 << 3;
        /// Left
        const LEFT = 1 << 4;
        /// Right
        const RIGHT = 1 << 5;

        /// Sequential
        const SEQUENTIAL = Self::NEXT.bits() | Self::PREVIOUS.bits();
        /// Vertical
        const VERTICAL = Self::UP.bits() | Self::DOWN.bits();
        /// Horizontal
        const HORIZONTAL = Self::LEFT.bits() | Self::RIGHT.bits();
        /// Directional
        const DIRECTIONAL = Self::VERTICAL.bits() | Self::HORIZONTAL.bits();
        /// All directions
        const ALL = Self::SEQUENTIAL.bits() | Self::DIRECTIONAL.bits();
    }
}
