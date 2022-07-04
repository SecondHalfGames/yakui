use std::any::TypeId;
use std::fmt;

use thunderdome::Index;

use crate::component::{Component, ErasedComponent, ErasedProps};

pub struct Element {
    pub type_id: TypeId,
    pub props: Box<dyn ErasedProps>,
    pub children: Vec<ElementId>,
    pub(crate) new: fn(Index, &dyn ErasedProps) -> Box<dyn ErasedComponent>,
}

impl Element {
    pub fn new<T: Component>(props: T::Props) -> Element {
        Element {
            type_id: TypeId::of::<T>(),
            props: Box::new(props),
            children: Vec::new(),
            new: crate::component::new::<T>,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(u32);

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

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("roots", &self.roots)
            .field("tree", &ViewTree(self))
            .finish()
    }
}

struct ViewTree<'a>(&'a Snapshot);

impl<'a> fmt::Debug for ViewTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dom = &self.0;
        let iter = dom.tree.iter().enumerate().map(|(index, element)| {
            let debug = element.props.as_debug();
            let children: Vec<_> = element.children.iter().collect();

            format!("{index:?}: {debug:?}, children: {children:?}")
        });

        f.debug_list().entries(iter).finish()
    }
}
