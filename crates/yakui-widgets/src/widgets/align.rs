use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Alignment, Response};

use crate::util::widget_children;

/**
Aligns its child according to the given alignment. Align should contain only one
child.

Responds with [AlignResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
yakui::center(|| {
    yakui::label("Centered!");
});

yakui::align(yakui::Alignment::BOTTOM_LEFT, || {
    yakui::label("Bottom left aligned");
});
```

Extended:
```rust
# let _handle = yakui_widgets::DocTest::start();
use yakui::widgets::Align;

Align::new(yakui::Alignment::TOP_RIGHT).show(|| {
    yakui::label("Top right aligned");
});
```
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Align {
    pub alignment: Alignment,
}

impl Align {
    pub fn new(alignment: Alignment) -> Self {
        Self { alignment }
    }

    pub fn center() -> Self {
        Self {
            alignment: Alignment::CENTER,
        }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<AlignResponse> {
        widget_children::<AlignWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct AlignWidget;

pub type AlignResponse = ();

impl Widget for AlignWidget {
    type Props = Align;
    type Response = AlignResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn layout(&self, props: &Self::Props, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        // Align allows its children to be smaller than the minimum size
        // enforced by the incoming constraints.
        let constraints = Constraints::loose(input.max);

        let mut self_size = if input.max.is_finite() {
            input.max
        } else if input.min.is_finite() {
            input.min
        } else {
            Vec2::ZERO
        };
        let align = props.alignment.as_vec2();

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            self_size = self_size.max(child_size);
            ctx.layout
                .set_pos(child, align * self_size - align * child_size);
        }

        self_size
    }

    fn paint(&self, _props: &Self::Props, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
