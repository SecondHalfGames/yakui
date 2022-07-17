//! Defines yakui's DOM, which holds the hierarchy of widgets and their
//! implementation details.

mod debug;
mod dummy;
mod root;

use std::any::{type_name, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::mem::replace;

use anymap::AnyMap;
use thunderdome::Arena;

use crate::id::WidgetId;
use crate::response::Response;
use crate::widget::{ErasedWidget, Widget};

use self::dummy::DummyWidget;
use self::root::RootWidget;

/// The DOM that contains the tree of active widgets.
pub struct Dom {
    inner: RefCell<DomInner>,
    globals: RefCell<AnyMap>,
}

struct DomInner {
    nodes: Arena<DomNode>,
    root: WidgetId,
    stack: RefCell<Vec<WidgetId>>,
}

/// A node in the [`Dom`].
pub struct DomNode {
    /// The widget implementation. Only a subset of the methods from [`Widget`]
    /// are available without downcasting the widget first.
    pub widget: Box<dyn ErasedWidget>,

    /// The parent of this node, if it has one.
    pub parent: Option<WidgetId>,

    /// All of this node's children.
    pub children: Vec<WidgetId>,

    /// Used when building the tree. The index of the next child if a new child
    /// starts being built.
    next_child: usize,
}

impl Dom {
    /// Create a new, empty DOM.
    pub fn new() -> Self {
        Self {
            globals: RefCell::new(AnyMap::new()),
            inner: RefCell::new(DomInner::new()),
        }
    }

    /// Start the build phase for the DOM and bind it to the current thread.
    pub fn start(&self) {
        log::debug!("Dom::start()");

        let mut dom = self.inner.borrow_mut();

        let root = dom.root;
        let root = dom.nodes.get_mut(root.index()).unwrap();
        root.next_child = 0;
    }

    /// End the DOM's build phase.
    pub fn finish(&self) {
        log::debug!("Dom::finish()");
    }

    /// Gives the root widget in the DOM. This widget will always exist.
    pub fn root(&self) -> WidgetId {
        let dom = self.inner.borrow();
        dom.root
    }

    /// Enter the context of the given widget, pushing it onto the stack so that
    /// [`Dom::current`] will report the correct widget.
    pub(crate) fn enter(&self, id: WidgetId) {
        let dom = self.inner.borrow();
        dom.stack.borrow_mut().push(id);
    }

    /// Pop the given widget off of the traversal stack. Panics if the widget on
    /// top of the stack is not the one with the given ID.
    pub(crate) fn exit(&self, id: WidgetId) {
        let dom = self.inner.borrow();
        assert_eq!(dom.stack.borrow_mut().pop(), Some(id));
    }

    /// If the DOM is being built, tells which widget is currently being built.
    ///
    /// This method only gives valid results when called from inside a
    /// [`Widget`] lifecycle method.
    pub fn current(&self) -> WidgetId {
        let dom = self.inner.borrow();
        let stack = dom.stack.borrow();
        stack.last().copied().unwrap_or(dom.root)
    }

    /// Returns a reference to the current DOM node. See [`Dom::current`].
    pub fn get_current(&self) -> Ref<'_, DomNode> {
        let dom = self.inner.borrow();
        let index = dom.current_widget().index();

        Ref::map(dom, |dom| dom.nodes.get(index).unwrap())
    }

    /// Get the node with the given widget ID.
    pub fn get(&self, id: WidgetId) -> Option<Ref<'_, DomNode>> {
        let dom = self.inner.borrow();
        let index = id.index();

        if dom.nodes.contains(index) {
            Some(Ref::map(dom, |dom| dom.nodes.get(index).unwrap()))
        } else {
            None
        }
    }

    /// Get a mutable reference to the node with the given widget ID.
    pub fn get_mut(&self, id: WidgetId) -> Option<RefMut<'_, DomNode>> {
        let dom = self.inner.borrow_mut();
        let index = id.index();

        if dom.nodes.contains(index) {
            Some(RefMut::map(dom, |dom| dom.nodes.get_mut(index).unwrap()))
        } else {
            None
        }
    }

    /// Get a piece of DOM-global state or initialize it with the given
    /// function.
    ///
    /// This is intended for any state that is global. It's not a perfect fit
    /// for scoped state like themes.
    pub fn get_global_or_init<T, F>(&self, init: F) -> T
    where
        T: 'static + Clone,
        F: FnOnce() -> T,
    {
        let mut globals = self.globals.borrow_mut();
        globals.entry::<T>().or_insert_with(init).clone()
    }

    /// Convenience method for calling [`Dom::begin_widget`] immediately
    /// followed by [`Dom::end_widget`].
    pub fn do_widget<T: Widget>(&self, props: T::Props) -> Response<T> {
        let index = self.begin_widget::<T>(props);
        self.end_widget::<T>(index)
    }

    /// Begin building a widget with the given type and props.
    ///
    /// After calling this method, children can be added to this widget.
    pub fn begin_widget<T: Widget>(&self, props: T::Props) -> WidgetId {
        log::trace!("begin_widget::<{}>({props:#?}", type_name::<T>());

        let (id, widget) = {
            let mut dom = self.inner.borrow_mut();
            let id = dom.next_widget();
            dom.stack.borrow_mut().push(id);
            dom.update_widget::<T>(id, props);

            // Component::children needs mutable access to the DOM, so we need
            // to rip the widget out of the tree so we can release our lock.
            let node = dom.nodes.get_mut(id.index()).unwrap();
            let widget = replace(&mut node.widget, Box::new(DummyWidget));

            (id, widget)
        };

        // Give this widget a chance to create children to take advantage of
        // composition.
        widget.children();

        // Quick! Put the widget back, before anyone notices!
        {
            let mut dom = self.inner.borrow_mut();
            let node = dom.nodes.get_mut(id.index()).unwrap();
            node.widget = widget;
        }

        id
    }

    /// Finish building the widget with the given ID. Must be the top of the
    /// stack, with no other widgets pending.
    ///
    /// Returns the widget's response type, wrapped in a [`Response`].
    pub fn end_widget<T: Widget>(&self, id: WidgetId) -> Response<T> {
        log::trace!("end_widget::<{}>({id:?})", type_name::<T>());

        let mut dom = self.inner.borrow_mut();

        let old_top = dom.stack.borrow_mut().pop().unwrap_or_else(|| {
            panic!("Cannot end_widget without an in-progress widget.");
        });

        assert!(
            id == old_top,
            "Dom::end_widget did not match the input widget."
        );

        dom.trim_children(id);

        let node = dom.nodes.get_mut(id.index()).unwrap();
        let res = node.widget.as_mut().downcast_mut::<T>().unwrap().respond();
        Response::new(id, res)
    }
}

impl DomInner {
    fn new() -> Self {
        let mut nodes = Arena::new();
        let root = nodes.insert(DomNode {
            widget: Box::new(RootWidget),
            parent: None,
            children: Vec::new(),
            next_child: 0,
        });

        Self {
            nodes,
            root: WidgetId::new(root),
            stack: RefCell::new(Vec::new()),
        }
    }

    fn current_widget(&self) -> WidgetId {
        let stack = self.stack.borrow();
        stack.last().copied().unwrap_or(self.root)
    }

    fn next_widget(&mut self) -> WidgetId {
        let parent_id = self.current_widget();

        let parent = self.nodes.get_mut(parent_id.index()).unwrap();
        if parent.next_child < parent.children.len() {
            let id = parent.children[parent.next_child];
            parent.next_child += 1;
            id
        } else {
            let index = self.nodes.insert(DomNode {
                widget: Box::new(DummyWidget),
                parent: Some(parent_id),
                children: Vec::new(),
                next_child: 0,
            });

            let id = WidgetId::new(index);

            let parent = self.nodes.get_mut(parent_id.index()).unwrap();
            parent.children.push(id);
            parent.next_child += 1;
            id
        }
    }

    fn update_widget<T: Widget>(&mut self, id: WidgetId, props: T::Props) {
        let node = self.nodes.get_mut(id.index()).unwrap();

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
    fn trim_children(&mut self, id: WidgetId) {
        let node = self.nodes.get_mut(id.index()).unwrap();

        if node.next_child < node.children.len() {
            let mut queue: VecDeque<WidgetId> = VecDeque::new();
            let to_drop = &node.children[node.next_child..];
            queue.extend(to_drop);

            node.children.truncate(node.next_child);

            while let Some(child_id) = queue.pop_front() {
                let child = self.nodes.remove(child_id.index()).unwrap();
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

        while let Some((id, depth)) = visit.pop_back() {
            let indent = "  ".repeat(depth);
            let node = self.nodes.get(id.index()).unwrap();

            writeln!(output, "{indent}{id:?} ({:?})", &node.children).unwrap();

            for &child in node.children.iter().rev() {
                visit.push_back((child, depth + 1));
            }
        }

        output
    }
}
