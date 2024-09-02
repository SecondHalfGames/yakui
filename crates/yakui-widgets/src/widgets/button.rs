use std::borrow::Cow;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::Color;
use yakui_core::input::MouseButton;
use yakui_core::widget::{EventContext, Widget};
use yakui_core::{Alignment, Response};

use crate::colors;
use crate::style::{TextAlignment, TextStyle};
use crate::util::widget;
use crate::widgets::Pad;

use super::{RenderText, RoundRect};

/**
A button containing some text.

Responds with [ButtonResponse].

Shorthand:
```rust
# let _handle = yakui_widgets::DocTest::start();
if yakui::button("Hello").clicked {
    println!("The button was clicked");
}
```
*/
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Button {
    pub text: Cow<'static, str>,
    pub padding: Pad,
    pub border_radius: f32,
    pub style: DynamicButtonStyle,
    pub hover_style: DynamicButtonStyle,
    pub down_style: DynamicButtonStyle,
}

/// Contains styles that can vary based on the state of the button.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DynamicButtonStyle {
    pub text: TextStyle,
    pub fill: Color,
}

impl Default for DynamicButtonStyle {
    fn default() -> Self {
        let mut text = TextStyle::label();
        text.align = TextAlignment::Center;

        Self {
            text,
            fill: Color::GRAY,
        }
    }
}

impl Button {
    pub fn unstyled(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            text: text.into(),
            padding: Pad::ZERO,
            border_radius: 0.0,
            style: DynamicButtonStyle::default(),
            hover_style: DynamicButtonStyle::default(),
            down_style: DynamicButtonStyle::default(),
        }
    }

    pub fn styled(text: impl Into<Cow<'static, str>>) -> Self {
        let style = DynamicButtonStyle {
            fill: colors::BACKGROUND_3,
            ..Default::default()
        };

        let hover_style = DynamicButtonStyle {
            fill: colors::BACKGROUND_3.adjust(1.2),
            ..Default::default()
        };

        let down_style = DynamicButtonStyle {
            fill: colors::BACKGROUND_3.adjust(0.8),
            ..Default::default()
        };

        let mut text_style = TextStyle::label();
        text_style.align = TextAlignment::Center;

        Self {
            text: text.into(),
            padding: Pad::balanced(20.0, 10.0),
            border_radius: 6.0,
            style,
            hover_style,
            down_style,
        }
    }

    pub fn show(self) -> Response<ButtonResponse> {
        widget::<ButtonWidget>(self)
    }
}

#[derive(Debug)]
pub struct ButtonWidget {
    props: Button,
    hovering: bool,
    mouse_down: bool,
    clicked: bool,
}

#[derive(Debug)]
pub struct ButtonResponse {
    pub hovering: bool,
    pub clicked: bool,
}

impl Widget for ButtonWidget {
    type Props<'a> = Button;
    type Response = ButtonResponse;

    fn new() -> Self {
        Self {
            props: Button::unstyled(Cow::Borrowed("")),
            hovering: false,
            mouse_down: false,
            clicked: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        let mut color = self.props.style.fill;
        let mut text_style = self.props.style.text.clone();

        if self.mouse_down {
            let style = &self.props.down_style;
            color = style.fill;
            text_style = style.text.clone();
        } else if self.hovering {
            let style = &self.props.hover_style;
            color = style.fill;
            text_style = style.text.clone();
        }

        let alignment = match text_style.align {
            TextAlignment::Start => Alignment::CENTER_LEFT,
            TextAlignment::Center => Alignment::CENTER,
            TextAlignment::End => Alignment::CENTER_RIGHT,
        };

        let mut container = RoundRect::new(self.props.border_radius);
        container.color = color;
        container.show_children(|| {
            crate::pad(self.props.padding, || {
                crate::align(alignment, || {
                    let mut text = RenderText::label(self.props.text.clone());
                    text.style = text_style;
                    text.show();
                });
            });
        });

        let clicked = self.clicked;
        self.clicked = false;

        Self::Response {
            hovering: self.hovering,
            clicked,
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
                EventResponse::Sink
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if *inside {
                    if *down {
                        self.mouse_down = true;
                        EventResponse::Sink
                    } else if self.mouse_down {
                        self.mouse_down = false;
                        self.clicked = true;
                        EventResponse::Sink
                    } else {
                        EventResponse::Bubble
                    }
                } else {
                    if !*down {
                        self.mouse_down = false;
                    }

                    EventResponse::Bubble
                }
            }
            _ => EventResponse::Bubble,
        }
    }
}
