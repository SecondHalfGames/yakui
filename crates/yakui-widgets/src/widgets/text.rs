use std::borrow::Cow;

use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::style::TextStyle;
use crate::util::widget;
use crate::{auto_builders, pad};

use super::{Pad, RenderText};

/**
Puts text onto the screen.

Responds with [TextResponse].

## Examples
```rust
# let _handle = yakui_widgets::DocTest::start();
# use yakui::widgets::Text;
yakui::label("Default text label style");

yakui::text(32.0, "Custom font size");

let mut text = Text::new(32.0, "Title");
text.style.color = yakui::Color::RED;
text.show();
```
*/
#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Text {
    pub text: Cow<'static, str>,
    pub style: TextStyle,
    pub padding: Pad,
}

auto_builders!(Text {
    style: TextStyle,
    padding: Pad,
});

impl Text {
    pub fn new<S: Into<Cow<'static, str>>>(font_size: f32, text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label().font_size(font_size),
            padding: Pad::ZERO,
        }
    }

    pub fn with_style<S: Into<Cow<'static, str>>>(text: S, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
            padding: Pad::ZERO,
        }
    }

    pub fn label(text: Cow<'static, str>) -> Self {
        Self {
            text,
            style: TextStyle::label(),
            padding: Pad::all(8.0),
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<TextResponse> {
        widget::<TextWidget>(self)
    }
}

#[derive(Debug)]
pub struct TextWidget {
    props: Text,
}

pub type TextResponse = ();

impl Widget for TextWidget {
    type Props<'a> = Text;
    type Response = TextResponse;

    fn new() -> Self {
        Self {
            props: Text::new(0.0, Cow::Borrowed("")),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        let mut render = RenderText::new(self.props.text.clone());
        render.style = self.props.style.clone();

        pad(self.props.padding, || {
            render.show();
        });
    }
}
