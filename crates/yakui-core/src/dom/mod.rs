mod debug;

use std::any::{Any, TypeId};
use std::collections::VecDeque;

use anymap::AnyMap;
use thunderdome::{Arena, Index};

use crate::component::{Component, DummyComponent, ErasedComponent};

pub struct Dom {
    tree: Arena<DomNode>,
    roots: Vec<Index>,

    color: bool,
    stack: Vec<Index>,
    build_index: usize,

    global_state: AnyMap,
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

            global_state: AnyMap::new(),
        }
    }

    pub fn start(&mut self) {
        self.color = !self.color;
        self.build_index = 0;
    }

    pub fn do_component<T: Component>(&mut self, props: T::Props) -> T::Response {
        let index = self.begin_component::<T>(props);
        self.end_component::<T>(index)
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
            let component = node.component.downcast_mut::<T>().unwrap();
            component.update(props);
        } else {
            node.component = Box::new(T::new(index, props));
        }

        index
    }

    pub fn end_component<T: Component>(&mut self, index: Index) -> T::Response {
        let old_top = self.stack.pop().unwrap_or_else(|| {
            panic!("Cannot end_component without an in-progress component.");
        });

        assert!(
            index == old_top,
            "Dom::end_component did not match the input component."
        );

        self.trim_children(index);

        let node = self.tree.get_mut(index).unwrap();

        node.component
            .as_mut()
            .downcast_mut::<T>()
            .unwrap()
            .respond()
    }

    pub fn get_global_state<T: Any>(&self) -> Option<&T> {
        self.global_state.get::<T>()
    }

    pub fn set_global_state<T: Any>(&mut self, value: T) -> Option<T> {
        self.global_state.insert::<T>(value)
    }

    pub fn get_global_state_or_insert_with<T: Any, F: FnOnce() -> T>(&mut self, init: F) -> &mut T {
        self.global_state.entry::<T>().or_insert_with(init)
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
