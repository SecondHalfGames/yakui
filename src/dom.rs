use std::collections::VecDeque;

use thunderdome::{Arena, Index};

use crate::component::Component;
use crate::snapshot::{ElementId, Snapshot};
use crate::zip_longest::zip;

pub struct Dom {
    tree: Arena<Node>,
    roots: Vec<Index>,
    snapshot: Option<Snapshot>,
}

struct Node {
    color: bool,
    children: Vec<Index>,
    component: Box<dyn Component>,
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
        let mut queue: VecDeque<(Option<ElementId>, Option<Index>)> = VecDeque::new();

        let snapshot_roots = snapshot.roots.iter().copied();
        let dom_roots = self.roots.iter().copied();
        queue.extend(zip(snapshot_roots, dom_roots));

        // Modifications
        while let Some((element_id, dom_index)) = queue.pop_front() {
            match (element_id, dom_index) {
                // Updated
                (Some(element_id), Some(dom_index)) => {
                    let element = snapshot.get(element_id).unwrap();
                    let dom_node = self.tree.get_mut(dom_index).unwrap();

                    // TODO: Check types?
                    dom_node.component.update(&element.props);

                    // Zip the children together and queue them for processing.
                    let element_children = element.children.iter().copied();
                    let dom_node_children = dom_node.children.iter().copied();
                    queue.extend(zip(element_children, dom_node_children));
                }

                // Added
                (Some(element_id), None) => {
                    let element = snapshot.get(element_id).unwrap();

                    // TODO: Contruction of components

                    // Queue all of the element's children for addition.
                    queue.extend(element.children.iter().copied().map(|id| (Some(id), None)));
                }

                // Removed
                (None, Some(dom_index)) => {
                    let node = self.tree.remove(dom_index).unwrap();

                    // Queue all of this node's children for removal.
                    queue.extend(node.children.into_iter().map(|index| (None, Some(index))));
                }

                (None, None) => unreachable!(),
            }
        }

        self.snapshot = Some(snapshot);
    }

    pub fn layout(&mut self) {
        todo!()
    }
}
