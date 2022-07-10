use std::collections::VecDeque;

use glam::Vec2;
use thunderdome::{Arena, Index};

use crate::dom::Dom;
use crate::geometry::{Constraints, Rect};

#[derive(Debug)]
pub struct LayoutDom {
    pub viewport: Rect,
    nodes: Arena<LayoutDomNode>,
}

#[derive(Debug)]
pub struct LayoutDomNode {
    pub rect: Rect,
}

impl LayoutDom {
    pub fn new() -> Self {
        Self {
            viewport: Rect::ZERO,
            nodes: Arena::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn get(&self, index: Index) -> Option<&LayoutDomNode> {
        self.nodes.get(index)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut LayoutDomNode> {
        self.nodes.get_mut(index)
    }

    pub fn calculate_all(&mut self, dom: &Dom) {
        let constraints = Constraints {
            min: Vec2::ZERO,
            max: self.viewport.size(),
        };

        for &index in dom.roots() {
            self.calculate(dom, index, constraints);
        }

        self.resolve_positions(dom);
    }

    pub fn calculate(&mut self, dom: &Dom, index: Index, constraints: Constraints) -> Vec2 {
        let dom_node = dom.get(index).unwrap();
        let size = dom_node.widget.layout(dom, self, constraints);
        self.nodes.insert_at(
            index,
            LayoutDomNode {
                rect: Rect::from_pos_size(Vec2::ZERO, size),
            },
        );
        size
    }

    pub fn set_pos(&mut self, index: Index, pos: Vec2) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.rect.set_pos(pos);
        }
    }

    fn resolve_positions(&mut self, dom: &Dom) {
        let mut queue = VecDeque::new();

        queue.extend(dom.roots().iter().map(|&index| (index, Vec2::ZERO)));

        while let Some((index, parent_pos)) = queue.pop_front() {
            if let Some(layout_node) = self.nodes.get_mut(index) {
                let node = dom.get(index).unwrap();
                layout_node
                    .rect
                    .set_pos(layout_node.rect.pos() + parent_pos);

                queue.extend(
                    node.children
                        .iter()
                        .map(|&index| (index, layout_node.rect.pos())),
                );
            }
        }
    }
}
