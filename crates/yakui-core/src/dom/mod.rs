mod debug;

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;

use anymap::AnyMap;
use thunderdome::{Arena, Index};

use crate::widget::{DummyWidget, ErasedWidget, Widget};

pub struct Dom {
    inner: RefCell<DomInner>,
}

struct DomInner {
    nodes: Arena<DomNode>,
    roots: Vec<Index>,

    color: bool,
    stack: Vec<Index>,
    build_index: usize,

    global_state: AnyMap,
}

pub struct DomNode {
    pub widget: Box<dyn ErasedWidget>,
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
            inner: RefCell::new(DomInner::new()),
        }
    }

    pub fn start(&self) {
        let mut dom = self.inner.borrow_mut();

        dom.color = !dom.color;
        dom.build_index = 0;
    }

    pub fn roots(&self) -> Ref<'_, [Index]> {
        let dom = self.inner.borrow();

        Ref::map(dom, |dom| dom.roots.as_slice())
    }

    pub fn get(&self, index: Index) -> Option<Ref<'_, DomNode>> {
        let dom = self.inner.borrow();

        if dom.nodes.contains(index) {
            Some(Ref::map(dom, |dom| dom.nodes.get(index).unwrap()))
        } else {
            None
        }
    }

    pub fn get_mut(&self, index: Index) -> Option<RefMut<'_, DomNode>> {
        let dom = self.inner.borrow_mut();

        if dom.nodes.contains(index) {
            Some(RefMut::map(dom, |dom| dom.nodes.get_mut(index).unwrap()))
        } else {
            None
        }
    }

    pub fn get_global_state_or_insert_with<T: Any, F: FnOnce() -> T>(
        &self,
        init: F,
    ) -> RefMut<'_, T> {
        let dom = self.inner.borrow_mut();

        RefMut::map(dom, |dom| {
            dom.global_state.entry::<T>().or_insert_with(init)
        })
    }

    pub fn do_widget<T: Widget>(&self, props: T::Props) -> T::Response {
        let index = self.begin_widget::<T>(props);
        self.end_widget::<T>(index)
    }

    pub fn begin_widget<T: Widget>(&self, props: T::Props) -> Index {
        let mut dom = self.inner.borrow_mut();
        let dom = &mut *dom;

        let parent = dom.stack.last();

        let index = match parent {
            Some(&parent_index) => {
                let parent = dom.nodes.get_mut(parent_index).unwrap();
                parent.set_color(dom.color);

                if parent.build_index < parent.children.len() {
                    let index = parent.children[parent.build_index];
                    parent.build_index += 1;
                    index
                } else {
                    let index = dom.nodes.insert(DomNode {
                        widget: Box::new(DummyWidget),
                        children: Vec::new(),
                        build_index: 0,
                        color: dom.color,
                    });

                    let parent = dom.nodes.get_mut(parent_index).unwrap();
                    parent.children.push(index);
                    parent.build_index += 1;
                    index
                }
            }
            None => {
                if dom.build_index < dom.roots.len() {
                    let index = dom.roots[dom.build_index];
                    dom.build_index += 1;
                    index
                } else {
                    let index = dom.nodes.insert(DomNode {
                        widget: Box::new(DummyWidget),
                        children: Vec::new(),
                        build_index: 0,
                        color: dom.color,
                    });
                    dom.roots.push(index);
                    dom.build_index += 1;
                    index
                }
            }
        };

        dom.stack.push(index);
        dom.update_widget::<T>(index, props);

        index
    }

    pub fn end_widget<T: Widget>(&self, index: Index) -> T::Response {
        let mut dom = self.inner.borrow_mut();

        let old_top = dom.stack.pop().unwrap_or_else(|| {
            panic!("Cannot end_widget without an in-progress widget.");
        });

        assert!(
            index == old_top,
            "Dom::end_widget did not match the input widget."
        );

        dom.trim_children(index);

        let node = dom.nodes.get_mut(index).unwrap();

        node.widget.as_mut().downcast_mut::<T>().unwrap().respond()
    }
}

impl DomInner {
    fn new() -> Self {
        Self {
            nodes: Arena::new(),
            roots: Vec::new(),

            color: false,
            stack: Vec::new(),
            build_index: 0,

            global_state: AnyMap::new(),
        }
    }

    fn update_widget<T: Widget>(&mut self, index: Index, props: T::Props) {
        let node = self.nodes.get_mut(index).unwrap();

        if node.widget.as_ref().type_id() == TypeId::of::<T>() {
            let widget = node.widget.downcast_mut::<T>().unwrap();
            widget.update(props);
        } else {
            node.widget = Box::new(T::new(index, props));
        }
    }

    /// Remove children from the given node that weren't present in the latest
    /// traversal through the tree.
    fn trim_children(&mut self, index: Index) {
        let node = self.nodes.get_mut(index).unwrap();

        if node.build_index < node.children.len() {
            let mut queue = VecDeque::new();
            let to_drop = &node.children[node.build_index..];
            queue.extend(to_drop);

            node.children.truncate(self.build_index);

            while let Some(index) = queue.pop_front() {
                let node = self.nodes.remove(index).unwrap();
                queue.extend(node.children);
            }
        }
    }
}
