use std::collections::{BTreeSet, VecDeque};

use thunderdome::{Arena, Index};

use crate::component::Component;
use crate::snapshot::{ElementId, Snapshot};

pub struct Dom {
    tree: Arena<Node>,
    roots: BTreeSet<Index>,
    color: bool,
    last_snapshot: Option<Snapshot>,
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
            roots: BTreeSet::new(),
            color: false,
            last_snapshot: None,
        }
    }

    pub fn take_snapshot(&mut self) -> Option<Snapshot> {
        self.last_snapshot.take()
    }

    pub fn apply(&mut self, snapshot: Snapshot) {
        self.color = !self.color;

        let mut queue: VecDeque<ElementId> = VecDeque::new();

        // TODO: Parallel tree traversal; details don't matter right now.

        queue.extend(&snapshot.roots);

        while let Some(id) = queue.pop_front() {
            let node = self.tree.get_by_slot_mut(id.value());

            match node {
                Some((_index, node)) => {
                    node.color = self.color;

                    todo!("update")
                }
                None => todo!("create"),
            }
        }

        let mut remove = Vec::new();

        for (index, node) in &self.tree {
            if node.color != self.color {
                remove.push(index);
            }
        }

        for index in remove {
            self.tree.remove(index);
        }

        self.last_snapshot = Some(snapshot);
    }

    pub fn layout(&mut self) {
        todo!()
    }
}
