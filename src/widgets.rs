use glam::Vec2;
use thunderdome::Index;

use crate::component::{Component, ComponentEvent};
use crate::context::Context;
use crate::dom::{Dom, LayoutDom};
use crate::draw::{Mesh, Vertex};
use crate::geometry::{Color3, Constraints};
use crate::input::MouseButton;

#[derive(Debug)]
pub struct List {
    pub props: ListProps,
    pub index: Index,
}

#[derive(Debug, Clone)]
pub struct ListProps {
    pub direction: Direction,
    pub fill: Option<Color3>,
}

impl ListProps {
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Down,
            fill: None,
        }
    }

    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Right,
            fill: None,
        }
    }
}

impl Component for List {
    type Props = ListProps;
    type Response = ();

    fn new(index: Index, props: Self::Props) -> Self {
        Self { props, index }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        let dom_node = dom.get(self.index).unwrap();

        let mut size = Vec2::ZERO;

        for &child in &dom_node.children {
            let child_size = layout.calculate(dom, child, constraints);

            match self.props.direction {
                Direction::Down => {
                    size.x = size.x.max(child_size.x);
                    size.y += child_size.y;
                }

                Direction::Right => {
                    size.x += child_size.x;
                    size.y = size.y.max(child_size.y);
                }
            }
        }

        let mut pos = Vec2::ZERO;

        for &child in &dom_node.children {
            let child = layout.get_mut(child).unwrap();
            child.rect.set_pos(pos);

            match self.props.direction {
                Direction::Down => {
                    pos += Vec2::new(0.0, child.rect.size().y);
                }

                Direction::Right => {
                    pos += Vec2::new(child.rect.size().x, 0.0);
                }
            }
        }

        constraints.constrain(size)
    }

    fn draw(&self, dom: &Dom, layout: &LayoutDom, output: &mut crate::draw::Output) {
        let dom_node = dom.get(self.index).unwrap();

        for &child in &dom_node.children {
            let child_node = dom.get(child).unwrap();
            child_node.component.draw(dom, layout, output);
        }
    }

    fn respond(&mut self) -> Self::Response {}
}

#[derive(Debug)]
pub struct FixedSizeBox {
    pub index: Index,
    pub hovered: bool,
    pub mouse_down_inside: bool,
    pub greenified: bool,
    pub clicked: bool,
    pub props: FixedSizeBoxProps,
}

#[derive(Debug, Clone)]
pub struct FixedSizeBoxProps {
    pub size: Vec2,
    pub color: Color3,
}

#[derive(Debug)]
pub struct FixedSizeBoxResponse {
    pub clicked: bool,
}

impl Component for FixedSizeBox {
    type Props = FixedSizeBoxProps;
    type Response = FixedSizeBoxResponse;

    fn new(index: Index, props: Self::Props) -> Self {
        Self {
            index,
            hovered: false,
            mouse_down_inside: false,
            greenified: false,
            clicked: false,
            props,
        }
    }

    fn update(&mut self, props: &Self::Props) {
        self.props = props.clone();
    }

    fn size(&self, _dom: &Dom, _layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        constraints.constrain(self.props.size)
    }

    fn draw(&self, _dom: &Dom, layout: &LayoutDom, output: &mut crate::draw::Output) {
        let layout_node = layout.get(self.index).unwrap();
        let size = layout_node.rect.size();
        let pos = layout_node.rect.pos();

        let view_pos = (pos + layout.viewport.pos()) / layout.viewport.size();
        let view_size = size / layout.viewport.size();

        #[rustfmt::skip]
        let positions = [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0]
        ].map(Vec2::from);

        let mut color = self.props.color.as_vec4(1.0);

        if self.greenified {
            color = Color3::rgb(127, 255, 127).as_vec4(1.0);
        } else if self.mouse_down_inside {
            color *= 0.3;
        } else if self.hovered {
            color *= 0.65;
        }

        let vertices = positions
            .map(|vert| Vertex::new(vert * view_size + view_pos, vert, color))
            .into();

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2,
            3, 0, 2,
        ];

        output.meshes.push(Mesh { vertices, indices });
    }

    fn respond(&mut self) -> Self::Response {
        let clicked = self.clicked;
        self.clicked = false;

        Self::Response { clicked }
    }

    fn event(&mut self, event: &ComponentEvent) {
        match event {
            ComponentEvent::MouseEnter => {
                self.hovered = true;
            }
            ComponentEvent::MouseLeave => {
                self.hovered = false;
            }
            ComponentEvent::MouseButtonChangedInside(MouseButton::One, down) => {
                if *down {
                    self.mouse_down_inside = true;
                } else if self.mouse_down_inside {
                    self.mouse_down_inside = false;
                    self.greenified = !self.greenified;
                    self.clicked = true;
                }
            }
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Down,
    Right,
}

pub fn vertical<F: FnOnce()>(contents: F) {
    let context = Context::active();

    let index = context
        .borrow_mut()
        .dom_mut()
        .begin_component::<List>(ListProps::vertical());

    contents();

    context.borrow_mut().dom_mut().end_component::<List>(index);
}

pub fn horizontal<F: FnOnce()>(contents: F) {
    let context = Context::active();

    let index = context
        .borrow_mut()
        .dom_mut()
        .begin_component::<List>(ListProps::horizontal());

    contents();

    context.borrow_mut().dom_mut().end_component::<List>(index);
}

pub fn fsbox<S: Into<Vec2>, C: Into<Color3>>(size: S, color: C) -> FixedSizeBoxResponse {
    let context = Context::active();

    let size = size.into();
    let color = color.into();
    let index = context
        .borrow_mut()
        .dom_mut()
        .begin_component::<FixedSizeBox>(FixedSizeBoxProps { size, color });

    let mut context = context.borrow_mut();
    let dom = context.dom_mut();
    dom.end_component::<FixedSizeBox>(index)
}
