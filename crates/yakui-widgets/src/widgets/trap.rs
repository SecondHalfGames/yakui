use yakui_core::{
    navigation::{NavDirection, NavDirections},
    widget::{NavigateContext, Widget},
    Response, WidgetId,
};

use crate::util::widget_children;

/**
A widget that traps navigation within its descendants.
*/
#[derive(Debug, Clone, Copy)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Trap {
    pub directions: NavDirections,
}

impl Trap {
    pub const fn new(directions: NavDirections) -> Self {
        Self { directions }
    }

    #[track_caller]
    pub fn show(self, children: impl FnOnce()) -> Response<()> {
        widget_children::<TrapWidget, _>(children, self)
    }
}

#[derive(Debug)]
pub struct TrapWidget {
    props: Trap,
}

impl Widget for TrapWidget {
    type Props<'a> = Trap;
    type Response = ();

    fn new() -> Self {
        Self {
            props: Trap::new(NavDirections::ALL),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn navigate(&self, ctx: NavigateContext<'_>, direction: NavDirection) -> Option<WidgetId> {
        // first, navigate among descendants normally
        if let Some(target) = self.default_navigate(ctx, direction) {
            return Some(target);
        }

        let focus = ctx.input.focus()?;

        // if the focus originated inside, keep it there
        if self.props.directions.contains(direction.flag())
            && ctx.contains(ctx.dom.current(), focus)
        {
            return Some(focus);
        }

        // it originated outside
        None
    }
}
