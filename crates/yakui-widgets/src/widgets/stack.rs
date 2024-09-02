use yakui_core::{widget::Widget, Response};

use crate::util::widget_children;

/**
A [Stack] widget. This widget does nothing interesting on its own, but
when used "inside" other layouts, such as [List](crate::widgets::List),
it will stacks its own children, rather than following the layout of its own parent.
This internal layouting is just using yakui's default layout algorithm.

Responds with [StackResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
yakui::column(|| {
    yakui::label("on top");
    yakui::stack(|| {
        // this would compose, by being stacked,
        // to appear like "hello world", barring issues from spacing,
        // as opposed to continuing the columnar layout setup above.
        yakui::text(12.0, "hello");
        yakui::text(12.0, "      world");
    });
    yakui::label("on bottom");
});
```
*/
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Stack {}

impl Stack {
    /// Creates a new [Stack].
    pub fn new() -> Self {
        Self {}
    }

    /// Shows the [Stack] along with its children.
    pub fn show<F: FnOnce()>(self, children: F) -> Response<StackResponse> {
        widget_children::<StackWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct StackWidget;

pub type StackResponse = ();

impl Widget for StackWidget {
    type Props<'a> = Stack;

    type Response = StackResponse;

    fn new() -> Self {
        Self
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {
        // nothing here
    }
}
