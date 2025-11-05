use std::borrow::Cow;

use yakui_core::widget::Widget;
use yakui_core::Response;

use crate::style::TextStyle;
use crate::util::widget;
use crate::widgets::RenderTextResponse;
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
    /// The text normally fills up the entire available space if it doesn't go from left to right,
    /// marking it as inline would force it to calculate the text width instead.
    ///
    /// See also: [`inline` in RenderText][super::RenderText]
    pub inline: bool,
    pub min_width: f32,
}

auto_builders!(Text {
    style: TextStyle,
    padding: Pad,
    inline: bool,
    min_width: f32,
});

impl Text {
    pub fn new<S: Into<Cow<'static, str>>>(font_size: f32, text: S) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::label().font_size(font_size),
            padding: Pad::ZERO,
            inline: false,
            min_width: 0.0,
        }
    }

    pub fn with_style<S: Into<Cow<'static, str>>>(text: S, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
            padding: Pad::ZERO,
            inline: false,
            min_width: 0.0,
        }
    }

    pub fn label(text: Cow<'static, str>) -> Self {
        Self {
            text,
            style: TextStyle::label(),
            padding: Pad::all(8.0),
            inline: false,
            min_width: 0.0,
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

pub struct TextResponse {
    pub render_text: RenderTextResponse,
}

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

        let mut render_text = None;
        pad(self.props.padding, || {
            render_text = Some(
                RenderText::with_style(self.props.text.clone(), self.props.style.clone())
                    .inline(self.props.inline)
                    .min_width(self.props.min_width)
                    .show()
                    .into_inner(),
            );
        });
        TextResponse {
            render_text: render_text.unwrap(),
        }
    }
}
