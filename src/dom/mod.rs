mod debug;
mod layout;
mod reconciler;

use thunderdome::{Arena, Index};

use crate::component::ErasedComponent;
use crate::snapshot::Snapshot;

pub use layout::*;

pub struct Dom {
    tree: Arena<DomNode>,
    roots: Vec<Index>,
    snapshot: Option<Snapshot>,
}

pub struct DomNode {
    pub component: Box<dyn ErasedComponent>,
    pub children: Vec<Index>,
}

impl Dom {
    pub fn new() -> Self {
        Self {
            tree: Arena::new(),
            roots: Vec::new(),
            snapshot: Some(Snapshot::new()),
        }
    }

    pub fn take_snapshot(&mut self) -> Option<Snapshot> {
        self.snapshot.take()
    }

    pub fn apply(&mut self, snapshot: Snapshot) {
        reconciler::apply(self, snapshot);
    }

    pub fn roots(&self) -> &[Index] {
        self.roots.as_slice()
    }

    pub fn get(&self, index: Index) -> Option<&DomNode> {
        self.tree.get(index)
    }
}
