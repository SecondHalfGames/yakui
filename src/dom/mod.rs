mod reconciler;

use std::any::Any;
use std::fmt;

use thunderdome::{Arena, Index};

use crate::registry::Registry;
use crate::snapshot::Snapshot;

pub struct Dom {
    tree: Arena<Node>,
    roots: Vec<Index>,
    snapshot: Option<Snapshot>,
    registry: Registry,
}

struct Node {
    component: Box<dyn Any>,
    children: Vec<Index>,
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

    pub fn _do_layout(&mut self) {
        todo!()
    }
}

impl fmt::Debug for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dom")
            .field("roots", &self.roots)
            .field("tree", &ViewTree(self))
            .field("snapshot", &self.snapshot)
            .finish()
    }
}

struct ViewTree<'a>(&'a Dom);

impl<'a> fmt::Debug for ViewTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dom = &self.0;
        let iter = dom.tree.iter().map(|(index, node)| {
            let id = node.component.as_ref().type_id();

            let debug = match dom.registry.get_by_id(id) {
                Some(component_impl) => (component_impl.debug)(node.component.as_ref()),
                None => &"(could not find debug impl)",
            };

            let children: Vec<_> = node.children.iter().map(|index| index.slot()).collect();

            format!("{}: {debug:?}, children: {:?}", index.slot(), children)
        });

        f.debug_list().entries(iter).finish()
    }
}
