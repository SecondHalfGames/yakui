mod debug;
mod layout;

use std::any::TypeId;
use std::collections::VecDeque;

use thunderdome::{Arena, Index};

use crate::component::{Component, DummyComponent, ErasedComponent};

pub use layout::*;

pub struct Dom {
    tree: Arena<DomNode>,
    roots: Vec<Index>,

    color: bool,
    stack: Vec<Index>,
    build_index: usize,
}

pub struct DomNode {
    pub component: Box<dyn ErasedComponent>,
    pub children: Vec<Index>,
    build_index: usize,
    color: bool,
}

impl DomNode {
    pub fn set_color(&mut self, color: bool) {
        if self.color != color {
            self.build_index = 0;
        }

        self.color = color;
    }
}

impl Dom {
    pub fn new() -> Self {
        Self {
            tree: Arena::new(),
            roots: Vec::new(),

            color: false,
            stack: Vec::new(),
            build_index: 0,
        }
    }

    pub fn start(&mut self) {
        self.color = !self.color;
        self.build_index = 0;
    }

    pub fn begin_component<T: Component>(&mut self, props: T::Props) -> Index {
        let parent = self.stack.last();

        let index = match parent {
            Some(&parent_index) => {
                let parent = self.tree.get_mut(parent_index).unwrap();
                parent.set_color(self.color);

                if parent.build_index < parent.children.len() {
                    let index = parent.children[parent.build_index];
                    parent.build_index += 1;
                    index
                } else {
                    let index = self.tree.insert(DomNode {
                        component: Box::new(DummyComponent),
                        children: Vec::new(),
                        build_index: 0,
                        color: self.color,
                    });

                    let parent = self.tree.get_mut(parent_index).unwrap();
                    parent.children.push(index);
                    parent.build_index += 1;
                    index
                }
            }
            None => {
                if self.build_index < self.roots.len() {
                    let index = self.roots[self.build_index];
                    self.build_index += 1;
                    index
                } else {
                    let index = self.tree.insert(DomNode {
                        component: Box::new(DummyComponent),
                        children: Vec::new(),
                        build_index: 0,
                        color: self.color,
                    });
                    self.roots.push(index);
                    self.build_index += 1;
                    index
                }
            }
        };

        self.stack.push(index);

        let node = self.tree.get_mut(index).unwrap();
        if node.component.as_ref().type_id() == TypeId::of::<T>() {
            node.component.update(&props);
        } else {
            node.component = Box::new(T::new(index, props));
        }

        index
    }

    pub fn end_component<T: Component>(&mut self, index: Index) -> T::Response {
        match self.stack.pop() {
            Some(old_top) => {
                assert!(
                    index == old_top,
                    "Dom::end_component did not match the input component."
                );

                self.trim_children(index);

                let node = self.tree.get(index).unwrap();

                node.component
                    .as_ref()
                    .downcast_ref::<T>()
                    .unwrap()
                    .respond()
            }
            None => {
                panic!("Cannot end_component without an in-progress component.");
            }
        }
    }

    /// Remove children from the given node that weren't present in the latest
    /// traversal through the tree.
    fn trim_children(&mut self, index: Index) {
        let node = self.tree.get_mut(index).unwrap();

        if node.build_index < node.children.len() {
            let mut queue = VecDeque::new();
            let to_drop = &node.children[node.build_index..];
            queue.extend(to_drop);

            node.children.truncate(self.build_index);

            while let Some(index) = queue.pop_front() {
                let node = self.tree.remove(index).unwrap();
                queue.extend(node.children);
            }
        }
    }

    pub fn roots(&self) -> &[Index] {
        self.roots.as_slice()
    }

    pub fn get(&self, index: Index) -> Option<&DomNode> {
        self.tree.get(index)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut DomNode> {
        self.tree.get_mut(index)
    }
}
