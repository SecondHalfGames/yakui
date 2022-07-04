use glam::Vec2;
use thunderdome::{Arena, Index};

use crate::layout::Constraints;

use super::Dom;

#[derive(Debug)]
pub struct LayoutDom {
    nodes: Arena<DomSizeNode>,
}

#[derive(Debug)]
pub struct DomSizeNode {
    pub pos: Vec2,
    pub size: Vec2,
}

impl LayoutDom {
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn get(&self, index: Index) -> Option<&DomSizeNode> {
        self.nodes.get(index)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut DomSizeNode> {
        self.nodes.get_mut(index)
    }

    pub fn calculate_all(&mut self, dom: &Dom, constraints: Constraints) {
        for &index in &dom.roots {
            self.calculate(dom, index, constraints);
        }
    }

    pub fn calculate(&mut self, dom: &Dom, index: Index, constraints: Constraints) -> Vec2 {
        let dom_node = dom.tree.get(index).unwrap();
        let size = dom_node.component.size(dom, self, constraints);
        self.nodes.insert_at(
            index,
            DomSizeNode {
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
}
