mod debug;
mod dummy;
mod root;

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::mem::replace;

use anymap::AnyMap;
use thunderdome::{Arena, Index};

use crate::widget::{ErasedWidget, Widget};

use self::dummy::DummyWidget;
use self::root::RootWidget;

pub struct Dom {
    inner: RefCell<DomInner>,
}

struct DomInner {
    nodes: Arena<DomNode>,
    root: Index,
    stack: RefCell<Vec<Index>>,
    global_state: AnyMap,
}

pub struct DomNode {
    pub widget: Box<dyn ErasedWidget>,
    pub children: Vec<Index>,
    next_child: usize,
}

impl Dom {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(DomInner::new()),
        }
    }

    pub fn start(&self) {
        let mut dom = self.inner.borrow_mut();

        let root = dom.root;
        let root = dom.nodes.get_mut(root).unwrap();
        root.next_child = 0;
    }

    pub fn root(&self) -> Index {
        let dom = self.inner.borrow();
        dom.root
    }

    pub(crate) fn enter(&self, widget: Index) {
        let dom = self.inner.borrow();
        dom.stack.borrow_mut().push(widget);
    }

    pub(crate) fn exit(&self, widget: Index) {
        let dom = self.inner.borrow();
        assert_eq!(dom.stack.borrow_mut().pop(), Some(widget));
    }

    pub fn current(&self) -> Index {
        let dom = self.inner.borrow();
        let stack = dom.stack.borrow();
        stack.last().copied().unwrap_or(dom.root)
    }

    pub fn get_current(&self) -> Ref<'_, DomNode> {
        let dom = self.inner.borrow();
        let index = dom.current_widget();

        Ref::map(dom, |dom| dom.nodes.get(index).unwrap())
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
        let (index, widget) = {
            let mut dom = self.inner.borrow_mut();
            let index = dom.next_widget();
            dom.stack.borrow_mut().push(index);
            dom.update_widget::<T>(index, props);

            // Component::children needs mutable access to the DOM, so we need
            // to rip the widget out of the tree so we can release our lock.
            let node = dom.nodes.get_mut(index).unwrap();
            let widget = replace(&mut node.widget, Box::new(DummyWidget));

            (index, widget)
        };

        // Give this widget a chance to create children to take advantage of
        // composition.
        widget.children();

        // Quick! Put the widget back, before anyone notices!
        {
            let mut dom = self.inner.borrow_mut();
            let node = dom.nodes.get_mut(index).unwrap();
            node.widget = widget;
        }

        index
    }

    pub fn end_widget<T: Widget>(&self, index: Index) -> T::Response {
        let mut dom = self.inner.borrow_mut();

        let old_top = dom.stack.borrow_mut().pop().unwrap_or_else(|| {
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
        let mut nodes = Arena::new();
        let root = nodes.insert(DomNode {
            widget: Box::new(DummyWidget),
            children: Vec::new(),
            next_child: 0,
        });

        nodes.get_mut(root).unwrap().widget = Box::new(RootWidget);

        Self {
            nodes,
            root,
            stack: RefCell::new(Vec::new()),
            global_state: AnyMap::new(),
        }
    }

    fn current_widget(&self) -> Index {
        let stack = self.stack.borrow();
        stack.last().copied().unwrap_or(self.root)
    }

    fn next_widget(&mut self) -> Index {
        let parent_index = self.current_widget();

        let parent = self.nodes.get_mut(parent_index).unwrap();
        if parent.next_child < parent.children.len() {
            let index = parent.children[parent.next_child];
            parent.next_child += 1;
            index
        } else {
            let index = self.nodes.insert(DomNode {
                widget: Box::new(DummyWidget),
                children: Vec::new(),
                next_child: 0,
            });

            let parent = self.nodes.get_mut(parent_index).unwrap();
            parent.children.push(index);
            parent.next_child += 1;
            index
        }
    }

    fn update_widget<T: Widget>(&mut self, index: Index, props: T::Props) {
        let node = self.nodes.get_mut(index).unwrap();

        if node.widget.as_ref().type_id() == TypeId::of::<T>() {
            let widget = node.widget.downcast_mut::<T>().unwrap();
            widget.update(props);
        } else {
            node.widget = Box::new(T::new(props));
        }

        node.next_child = 0;
    }

    /// Remove children from the given node that weren't present in the latest
    /// traversal through the tree.
    fn trim_children(&mut self, index: Index) {
        let node = self.nodes.get_mut(index).unwrap();

        if node.next_child < node.children.len() {
            let mut queue = VecDeque::new();
            let to_drop = &node.children[node.next_child..];
            queue.extend(to_drop);

            node.children.truncate(node.next_child);

            while let Some(child_index) = queue.pop_front() {
                let child = self.nodes.remove(child_index).unwrap();
                queue.extend(child.children);
            }
        }
    }

    #[allow(unused)]
    fn debug_tree(&self) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        let mut visit = VecDeque::new();
        visit.push_back((self.root, 0));

        while let Some((index, depth)) = visit.pop_back() {
            let indent = "  ".repeat(depth);
            let node = self.nodes.get(index).unwrap();
            let slot = index.slot();
            let children: Vec<_> = node.children.iter().map(|child| child.slot()).collect();

            writeln!(output, "{indent}{slot} ({children:?})").unwrap();

            for &child in node.children.iter().rev() {
                visit.push_back((child, depth + 1));
            }
        }

        output
    }
}
