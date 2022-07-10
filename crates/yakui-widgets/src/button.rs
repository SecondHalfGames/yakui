use std::borrow::Cow;

use yakui_core::paint::PaintDom;
use yakui_core::{
    dom::Dom, layout::LayoutDom, Color3, Constraints, MouseButton, Vec2, Widget, WidgetEvent,
};

use crate::util::widget;
use crate::Padding;

#[derive(Debug, Clone)]
pub struct Button {
    pub text: Cow<'static, str>,
    pub padding: Padding,
    pub fill: Color3,
    pub hover_fill: Option<Color3>,
    pub down_fill: Option<Color3>,
}

impl Button {
    pub fn unstyled<S: Into<Cow<'static, str>>>(text: S) -> Self {
        Self {
            text: text.into(),
            padding: Padding::even(0.0),
            fill: Color3::GRAY,
            hover_fill: None,
            down_fill: None,
        }
    }

    pub fn styled<S: Into<Cow<'static, str>>>(text: S) -> Self {
        Self {
            text: text.into(),
            padding: Padding::even(6.0),
            fill: Color3::rgb(50, 94, 168),
            hover_fill: Some(Color3::rgb(88, 129, 199)),
            down_fill: Some(Color3::rgb(30, 76, 156)),
        }
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

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        let node = dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = layout.calculate(dom, child, constraints);
            size = size.max(child_size);
        }

        constraints.constrain(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }

    fn respond(&mut self) -> Self::Response {
        let clicked = self.clicked;
        self.clicked = false;

        Self::Response {
            hovering: self.hovering,
            clicked,
        }
    }

    fn event(&mut self, event: &WidgetEvent) {
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
            }
            WidgetEvent::MouseButtonChangedInside(MouseButton::One, down) => {
                if *down {
                    self.mouse_down = true;
                } else if self.mouse_down {
                    self.mouse_down = false;
                    self.clicked = true;
                }
            }
            _ => {}
        }
    }
}

pub fn button(props: Button) -> ButtonResponse {
    widget::<ButtonWidget>(props)
}
