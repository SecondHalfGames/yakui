use yakui_core::paint::{PaintDom, PaintRect};
use yakui_core::Rect;
use yakui_core::{
    dom::Dom, layout::LayoutDom, Color3, Constraints, Index, MouseButton, Vec2, Widget, WidgetEvent,
};

use crate::util::widget;

#[derive(Debug, Clone)]
pub struct Button {
    pub size: Vec2,
    pub fill: Color3,
    pub hover_fill: Option<Color3>,
    pub down_fill: Option<Color3>,
}

impl Button {
    pub fn unstyled<S: Into<Vec2>>(size: S) -> Self {
        Self {
            size: size.into(),
            fill: Color3::GRAY,
            hover_fill: None,
            down_fill: None,
        }
    }

    pub fn styled<S: Into<Vec2>>(size: S) -> Self {
        Self {
            size: size.into(),
            fill: Color3::rgb(50, 94, 168),
            hover_fill: Some(Color3::rgb(88, 129, 199)),
            down_fill: Some(Color3::rgb(30, 76, 156)),
        }
    }
}

#[derive(Debug)]
pub struct ButtonWidget {
    index: Index,
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

    fn new(index: Index, props: Self::Props) -> Self {
        Self {
            index,
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
        crate::text(8.0, "hey, listen!");
    }

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        let node = dom.get(self.index).unwrap();
        for &child in &node.children {
            layout.calculate(dom, child, constraints);
        }

        constraints.constrain(self.props.size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = layout.get(self.index).unwrap();
        let viewport = layout.viewport();
        let size = node.rect.size() / viewport.size();
        let pos = (node.rect.pos() + viewport.pos()) / viewport.size();

        let mut color = self.props.fill;

        if let (Some(fill), true) = (self.props.down_fill, self.mouse_down) {
            color = fill
        } else if let (Some(hover), true) = (self.props.hover_fill, self.hovering) {
            color = hover
        }

        let mut rect = PaintRect::new(Rect::from_pos_size(pos, size));
        rect.color = color;
        paint.add_rect(rect);

        let node = dom.get(self.index).unwrap();
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
