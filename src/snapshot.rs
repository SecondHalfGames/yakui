use std::any::{Any, TypeId};

pub struct Element {
    pub type_id: TypeId,
    pub props: Box<dyn Any>,
    pub children: Vec<ElementId>,
}

impl Element {
    pub fn new<T: Any, P: Any>(props: P) -> Element {
        Element {
            type_id: TypeId::of::<T>(),
            props: Box::new(props),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(u32);

impl ElementId {
    pub fn value(self) -> u32 {
        self.0
    }
}

pub struct Snapshot {
    pub tree: Vec<Element>,
    pub roots: Vec<ElementId>,
    pub stack: Vec<ElementId>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            roots: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tree.clear();
        self.roots.clear();
        self.stack.clear();
    }

    pub fn get(&self, id: ElementId) -> Option<&Element> {
        self.tree.get(id.0 as usize)
    }

    pub(crate) fn insert(&mut self, element: Element) -> ElementId {
        let id = ElementId(self.tree.len() as u32);

        if let Some(top) = self.stack.last() {
            let top_element = &mut self.tree[top.0 as usize];
            top_element.children.push(id);
        } else {
            self.roots.push(id);
        }

        self.tree.push(element);

        id
    }

    pub(crate) fn push(&mut self, element: Element) -> ElementId {
        let id = self.insert(element);
        self.stack.push(id);
        id
    }

    pub(crate) fn pop(&mut self, id: ElementId) {
        match self.stack.pop() {
            Some(old_top) => {
                assert!(id == old_top, "Snapshot::pop popped the wrong element!");
            }
            None => {
                panic!("Cannot pop when there are no elements on the stack.");
            }
        }
    }
}
