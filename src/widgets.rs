use glam::{Vec2, Vec4};
use thunderdome::Index;

use crate::component::Component;
use crate::context::Context;
use crate::dom::{Dom, LayoutDom};
use crate::draw::{Mesh, Vertex};
use crate::layout::Constraints;
use crate::snapshot::Element;

#[derive(Debug)]
pub struct List {
    pub props: ListProps,
    pub index: Index,
}

#[derive(Debug, Clone)]
pub struct ListProps {
    pub direction: Direction,
}

impl ListProps {
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Down,
        }
    }
}

impl Component for List {
    type Props = ListProps;

    fn new(index: Index, props: &Self::Props) -> Self {
        Self {
            props: props.clone(),
            index,
        }
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
            child.pos = pos;

            match self.props.direction {
                Direction::Down => {
                    pos += Vec2::new(0.0, child.size.y);
                }

                Direction::Right => {
                    pos += Vec2::new(child.size.x, 0.0);
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
}

#[derive(Debug)]
pub struct FixedSizeBox {
    pub index: Index,
    pub props: FixedSizeBoxProps,
}

#[derive(Debug, Clone)]
pub struct FixedSizeBoxProps {
    pub size: Vec2,
}

impl Component for FixedSizeBox {
    type Props = FixedSizeBoxProps;

    fn new(index: Index, props: &Self::Props) -> Self {
        Self {
            index,
            props: props.clone(),
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
        let size = layout_node.size;
        let pos = layout_node.pos;

        let view_pos = (pos + layout.viewport.pos()) / layout.viewport.size();
        let view_size = size / layout.viewport.size();

        #[rustfmt::skip]
        let positions = [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0]
        ].map(Vec2::from);

        let vertices = positions
            .map(|vert| {
                Vertex::new(
                    vert * view_size + view_pos,
                    vert,
                    Vec4::new(vert.x, vert.y, 0.0, 1.0),
                )
            })
            .into();

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2,
            3, 0, 2,
        ];

        output.meshes.push(Mesh { vertices, indices });
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Down,
    Right,
}

pub fn vertical<F: FnOnce()>(contents: F) {
    let context = Context::active();

    let id = context
        .borrow_mut()
        .snapshot_mut()
        .push(Element::new::<List, _>(ListProps::vertical()));

    contents();

    context.borrow_mut().snapshot_mut().pop(id);
}

pub fn fsbox<S: Into<Vec2>>(size: S) {
    let context = Context::active();

    let size = size.into();
    context
        .borrow_mut()
        .snapshot_mut()
        .insert(Element::new::<FixedSizeBox, _>(FixedSizeBoxProps { size }));
}
