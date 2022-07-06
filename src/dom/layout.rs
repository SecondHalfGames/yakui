use std::collections::VecDeque;

use glam::Vec2;
use thunderdome::{Arena, Index};

use crate::geometry::{Constraints, Rect};

use super::Dom;

#[derive(Debug)]
pub struct LayoutDom {
    pub viewport: Rect,
    nodes: Arena<LayoutDomNode>,
}

#[derive(Debug)]
pub struct LayoutDomNode {
    pub pos: Vec2,
    pub size: Vec2,
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
            min: None,
            max: Some(self.viewport.size()),
        };

        for &index in &dom.roots {
            self.calculate(dom, index, constraints);
        }

        self.resolve_positions(dom);
    }

    pub fn calculate(&mut self, dom: &Dom, index: Index, constraints: Constraints) -> Vec2 {
        let dom_node = dom.tree.get(index).unwrap();
        let size = dom_node.component.size(dom, self, constraints);
        self.nodes.insert_at(
            index,
            LayoutDomNode {
                size,
                pos: Vec2::ZERO,
            },
        );
        size
    }

    pub fn set_pos(&mut self, index: Index, pos: Vec2) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.pos = pos;
        }
    }

    fn resolve_positions(&mut self, dom: &Dom) {
        let mut queue = VecDeque::new();

        queue.extend(dom.roots.iter().map(|&index| (index, Vec2::ZERO)));

        while let Some((index, parent_pos)) = queue.pop_front() {
            if let Some(layout_node) = self.nodes.get_mut(index) {
                let node = dom.get(index).unwrap();
                layout_node.pos += parent_pos;

                queue.extend(node.children.iter().map(|&index| (index, layout_node.pos)));
            }
        }
    }
}
