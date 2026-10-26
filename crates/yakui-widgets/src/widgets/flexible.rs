use yakui_core::geometry::FlexFit;
use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::auto_builders;
use crate::util::widget_children;

/**
A container that returns configurable flex values.

Responds with [FlexibleResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
yakui::expanded(|| {
    yakui::button("This will expand to fill the container");
});

yakui::flexible(2, || {
    yakui::button("This will grow with a flex factor of 2");
});
```
*/
#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Flexible {
    pub flex: u32,
    pub fit: FlexFit,
}

auto_builders!(Flexible {
    flex: u32,
    fit: FlexFit,
});

impl Flexible {
    pub fn new(flex: u32) -> Self {
        Self {
            flex,
            fit: FlexFit::Loose,
        }
    }

    pub fn tight(flex: u32) -> Self {
        Self {
            flex,
            fit: FlexFit::Tight,
        }
    }

    pub fn expanded() -> Self {
        Self {
            flex: 1,
            fit: FlexFit::Tight,
        }
    }

    #[track_caller]
    pub fn show<F: FnOnce()>(self, children: F) -> Response<FlexibleResponse> {
        widget_children::<FlexibleWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct FlexibleWidget {
    props: Flexible,
}

pub type FlexibleResponse = ();

impl Widget for FlexibleWidget {
    type Props<'a> = Flexible;
    type Response = FlexibleResponse;

    fn new() -> Self {
        Self {
            props: Flexible::new(0),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        (self.props.flex, self.props.fit)
    }
}
