use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::auto_builders;
use crate::util::widget_children;

/**
Applies padding around a single child widget.

Responds with [PadResponse].
*/
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Pad {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

auto_builders!(Pad {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
});

impl Pad {
    pub const ZERO: Pad = Pad::all(0.0);

    pub const fn none() -> Self {
        Self::ZERO
    }

    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub const fn balanced(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    pub const fn horizontal(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: 0.0,
            bottom: 0.0,
        }
    }

    pub const fn vertical(value: f32) -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: value,
            bottom: value,
        }
    }

    pub const fn offset(&self) -> Vec2 {
        Vec2::new(self.left, self.top)
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<PadResponse> {
        widget_children::<PadWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct PadWidget {
    props: Pad,
}

pub type PadResponse = ();

impl Widget for PadWidget {
    type Props<'a> = Pad;
    type Response = PadResponse;

    fn new() -> Self {
        Self { props: Pad::ZERO }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        let total_padding = Vec2::new(
            self.props.left + self.props.right,
            self.props.top + self.props.bottom,
        );
        let offset = Vec2::new(self.props.left, self.props.top);

        let child_constraints = Constraints {
            min: (input.min - total_padding).max(Vec2::ZERO),
            max: (input.max - total_padding).max(Vec2::ZERO),
        };

        let mut self_size = Vec2::ZERO;

        for &child in &node.children {
            self_size = ctx.calculate_layout(child, child_constraints) + total_padding;
            ctx.layout.set_pos(child, offset);
        }

        self_size = self_size.max(total_padding);
        input.constrain_min(self_size)
    }
}
