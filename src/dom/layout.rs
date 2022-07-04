use glam::Vec2;
use thunderdome::{Arena, Index};

use crate::layout::{Constraints, Layout};

use super::Dom;

pub struct DomSize {
    nodes: Arena<DomSizeNode>,
}

pub struct DomSizeNode {
    size: Vec2,
}

impl DomSize {
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn calculate_all(&mut self, dom: &Dom) {}

    pub fn calculate(&mut self, dom: &Dom, index: Index, constraints: Constraints) {
        let dom_node = dom.tree.get(index).unwrap();
        let component_impl = dom
            .registry
            .get_by_id(dom_node.component.as_ref().type_id())
            .unwrap();

        let size = (component_impl.size)(dom_node.component.as_ref(), dom, constraints);
        self.nodes.insert_at(index, DomSizeNode { size });
    }
}

pub fn calculate(dom: &Dom) -> Layout {
    let constraints = Constraints {
        min: None,
        max: None,
    };

    for id in &dom.roots {}

    todo!()
}
