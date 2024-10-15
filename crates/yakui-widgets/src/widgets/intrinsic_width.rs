use yakui_core::dom::Dom;
use yakui_core::geometry::{Constraints, FlexFit, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;

use crate::util::widget_children;

/**
A container that sizes its child to the child's intrinsic width

Responds with [IntrinsicWidthResponse].

Shorthand:
```rust

```
 */
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct IntrinsicWidth {

}

impl IntrinsicWidth {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<IntrinsicWidthResponse> {
        widget_children::<IntrinsicWidthWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct IntrinsicWidthWidget {
    props: IntrinsicWidth,
}

pub type IntrinsicWidthResponse = ();

impl Widget for IntrinsicWidthWidget {
    type Props<'a> = IntrinsicWidth;
    type Response = IntrinsicWidthResponse;

    fn new() -> Self {
        Self {
            props: IntrinsicWidth::new(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        let intrinsic_width=self.intrinsic_width(&node,&ctx.dom);
        let constraints = Constraints {
            min: (input.min).max(Vec2::ZERO),
            max: (input.max).max(Vec2::ZERO).min(Vec2::new(intrinsic_width,input.max.y)),
        };

        let mut size = Vec2::ZERO;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        input.constrain(constraints.constrain(size))
    }


}
