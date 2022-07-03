mod debug;
mod layout;
mod reconciler;

use std::any::Any;

use glam::Vec2;
use thunderdome::{Arena, Index};

use crate::layout::Layout;
use crate::registry::Registry;
use crate::snapshot::Snapshot;
use crate::Constraints;

pub struct Dom {
    tree: Arena<DomNode>,
    roots: Vec<Index>,
    snapshot: Option<Snapshot>,
    registry: Registry,
}

pub struct DomNode {
    pub component: Box<dyn Any>,
    pub children: Vec<Index>,
}

impl Dom {
    pub fn new(registry: Registry) -> Self {
        Self {
            tree: Arena::new(),
            roots: Vec::new(),
            snapshot: Some(Snapshot::new(registry.clone())),
            registry,
        }
    }

    pub fn take_snapshot(&mut self) -> Option<Snapshot> {
        self.snapshot.take()
    }

    pub fn apply(&mut self, snapshot: Snapshot) {
        reconciler::apply(self, snapshot);
    }

    pub fn layout(&mut self) -> Layout {
        layout::calculate(self)
    }

    pub fn get(&self, index: Index) -> Option<&DomNode> {
        self.tree.get(index)
    }

    pub fn size(&self, index: Index, constraints: Constraints) -> Vec2 {
        let dom_node = self.tree.get(index).unwrap();
        let component_impl = self
            .registry
            .get_by_id(dom_node.component.as_ref().type_id())
            .unwrap();

        (component_impl.size)(&dom_node.component, self, constraints)
    }
}
