use std::borrow::Cow;

use yakui_core::input::MouseButton;
use yakui_core::widget::Widget;
use yakui_core::{Color3, EventInterest, EventResponse, Response, WidgetEvent};

use crate::colors;
use crate::util::widget;
use crate::Pad;

#[derive(Debug, Clone)]
pub struct Button {
    pub text: Cow<'static, str>,
    pub padding: Pad,
    pub fill: Color3,
    pub hover_fill: Option<Color3>,
    pub down_fill: Option<Color3>,
}

impl Button {
    pub fn unstyled(text: Cow<'static, str>) -> Self {
        Self {
            text,
            padding: Pad::ZERO,
            fill: Color3::GRAY,
            hover_fill: None,
            down_fill: None,
        }
    }

    pub fn styled(text: Cow<'static, str>) -> Self {
        Self {
            text,
            padding: Pad::balanced(20.0, 10.0),
            fill: colors::BACKGROUND_3,
            hover_fill: Some(colors::BACKGROUND_3.adjust(1.2)),
            down_fill: Some(colors::BACKGROUND_3.adjust(0.8)),
        }
    }

    pub fn show(self) -> Response<ButtonWidget> {
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
    type Props = Button;
    type Response = ButtonResponse;

    fn new(props: Self::Props) -> Self {
        Self {
            props,
            hovering: false,
            mouse_down: false,
            clicked: false,
        }
    }

    fn update(&mut self, props: Self::Props) {
        self.props = props;
    }

    fn children(&self) {
        let mut color = self.props.fill;

        if let (Some(fill), true) = (self.props.down_fill, self.mouse_down) {
            color = fill
        } else if let (Some(hover), true) = (self.props.hover_fill, self.hovering) {
            color = hover
        }

        crate::colored_box_container(color, || {
            crate::pad(self.props.padding, || {
                crate::text(16.0, self.props.text.clone());
            });
        });
    }

    fn respond(&mut self) -> Self::Response {
        let clicked = self.clicked;
        self.clicked = false;

        Self::Response {
            hovering: self.hovering,
            clicked,
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
                EventResponse::Sink
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged(MouseButton::One, down) => {
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
            }
            WidgetEvent::MouseButtonChangedOutside(MouseButton::One, down) => {
                if !*down {
                    self.mouse_down = false;
                }

                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
