use std::borrow::Cow;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::Color;
use yakui_core::input::MouseButton;
use yakui_core::widget::{EventContext, Widget};
use yakui_core::{Alignment, Response};

use crate::style::{TextAlignment, TextStyle};
use crate::util::widget;
use crate::widgets::Pad;
use crate::{auto_builders, colors};

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
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Button {
    pub text: Cow<'static, str>,
    pub alignment: Alignment,
    pub padding: Pad,
    pub top_left_radius: f32,
    pub top_right_radius: f32,
    pub bottom_left_radius: f32,
    pub bottom_right_radius: f32,
    pub style: DynamicButtonStyle,
    pub hover_style: DynamicButtonStyle,
    pub down_style: DynamicButtonStyle,
}

auto_builders!(Button {
    text: Cow<'static, str>,
    alignment: Alignment,
    padding: Pad,
    top_left_radius: f32,
    top_right_radius: f32,
    bottom_left_radius: f32,
    bottom_right_radius: f32,
    style: DynamicButtonStyle,
    hover_style: DynamicButtonStyle,
    down_style: DynamicButtonStyle,
});

/// Contains styles that can vary based on the state of the button.
#[derive(Debug, Clone)]
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
            alignment: Alignment::CENTER,
            padding: Pad::ZERO,
            top_left_radius: 0.0,
            top_right_radius: 0.0,
            bottom_left_radius: 0.0,
            bottom_right_radius: 0.0,
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

        Self {
            text: text.into(),
            alignment: Alignment::CENTER,
            padding: Pad::balanced(20.0, 10.0),
            top_left_radius: 6.0,
            top_right_radius: 6.0,
            bottom_left_radius: 6.0,
            bottom_right_radius: 6.0,
            style,
            hover_style,
            down_style,
        }
    }

    pub fn border_radius(mut self, radius: f32) -> Self {
        self.top_left_radius = radius;
        self.top_right_radius = radius;
        self.bottom_left_radius = radius;
        self.bottom_right_radius = radius;
        self
    }

    pub fn border_radii(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_left: f32,
        bottom_right: f32,
    ) -> Self {
        self.top_left_radius = top_left;
        self.top_right_radius = top_right;
        self.bottom_left_radius = bottom_left;
        self.bottom_right_radius = bottom_right;
        self
    }

    pub fn top_border_radius(mut self, radius: f32) -> Self {
        self.top_left_radius = radius;
        self.top_right_radius = radius;
        self
    }

    pub fn bottom_border_radius(mut self, radius: f32) -> Self {
        self.bottom_left_radius = radius;
        self.bottom_right_radius = radius;
        self
    }

    pub fn left_border_radius(mut self, radius: f32) -> Self {
        self.top_left_radius = radius;
        self.bottom_left_radius = radius;
        self
    }

    pub fn right_border_radius(mut self, radius: f32) -> Self {
        self.top_right_radius = radius;
        self.bottom_right_radius = radius;
        self
    }

    #[track_caller]
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

        let align = match text_style.align {
            TextAlignment::Start => Alignment::CENTER_LEFT,
            TextAlignment::Center => Alignment::CENTER,
            TextAlignment::End => Alignment::CENTER_RIGHT,
        };

        let mut container = RoundRect::new(0.0);
        container.top_left_radius = self.props.top_left_radius;
        container.top_right_radius = self.props.top_right_radius;
        container.bottom_left_radius = self.props.bottom_left_radius;
        container.bottom_right_radius = self.props.bottom_right_radius;
        container.color = color;
        container.show_children(|| {
            crate::pad(self.props.padding, || {
                crate::align(align, || {
                    RenderText::with_style(self.props.text.clone(), text_style).show();
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
