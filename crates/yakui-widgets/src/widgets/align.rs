use yakui_core::dom::Dom;
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::layout::LayoutDom;
use yakui_core::paint::PaintDom;
use yakui_core::widget::Widget;
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
#[non_exhaustive]
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

    pub fn show<F: FnOnce()>(self, children: F) -> Response<AlignWidget> {
        widget_children::<AlignWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct AlignWidget {
    props: Align,
}

pub type AlignResponse = ();

impl Widget for AlignWidget {
    type Props = Align;
    type Response = AlignResponse;

    fn new() -> Self {
        Self {
            props: Align::center(),
        }
    }

    fn update(&mut self, props: Self::Props) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, input: Constraints) -> Vec2 {
        let node = dom.get_current();

        // Align allows its children to be smaller than the minimum size
        // enforced by the incoming constraints.
        let constraints = Constraints::loose(input.max);

        // FIXME: Align should behave when given unbounded constraints.
        // Currently, this will cause Align's child to be pushed off the screen.
        let self_size = input.max;
        let align = self.props.alignment.as_vec2();

        for &child in &node.children {
            let child_size = layout.calculate(dom, child, constraints);
            layout.set_pos(child, align * self_size - align * child_size);
        }

        self_size
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }
}
