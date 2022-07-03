use std::any::Any;
use std::collections::VecDeque;
use std::fmt;

use thunderdome::{Arena, Index};

use crate::registry::Registry;
use crate::snapshot::{ElementId, Snapshot};
use crate::zip_longest::zip;

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
        struct WorkItem {
            element: Option<ElementId>,
            dom_node: Option<Index>,
            parent: Option<Index>,
        }

        let mut queue: VecDeque<WorkItem> = VecDeque::new();

        let snapshot_roots = snapshot.roots.iter().copied();
        let dom_roots = self.roots.iter().copied();
        let work = zip(snapshot_roots, dom_roots).map(|(element, dom_node)| WorkItem {
            element,
            dom_node,
            parent: None,
        });
        queue.extend(work);

        // Modifications
        while let Some(work_item) = queue.pop_front() {
            match (work_item.element, work_item.dom_node) {
                // Updated
                (Some(element_id), Some(dom_index)) => {
                    let element = snapshot.get(element_id).unwrap();
                    let dom_node = self.tree.get_mut(dom_index).unwrap();

                    if element.type_id == dom_node.component.as_ref().type_id() {
                        if let Some(component_impl) = self.registry.get_by_id(element.type_id) {
                            (component_impl.update)(
                                dom_node.component.as_mut(),
                                element.props.as_ref(),
                            );
                        } else {
                            panic!("Unknown component ID {:?}", element.type_id);
                        }
                    } else {
                        // Because this component has changed types, delete the
                        // old one and create a new one immediately after.
                        queue.push_front(WorkItem {
                            element: Some(element_id),
                            dom_node: None,
                            parent: work_item.parent,
                        });
                        queue.push_front(WorkItem {
                            element: None,
                            dom_node: Some(dom_index),
                            parent: work_item.parent,
                        });
                    }

                    // Zip the children together and queue them for processing.
                    let element_children = element.children.iter().copied();
                    let dom_node_children = dom_node.children.iter().copied();

                    let work =
                        zip(element_children, dom_node_children).map(|(element, dom_node)| {
                            WorkItem {
                                element,
                                dom_node,
                                parent: Some(dom_index),
                            }
                        });
                    queue.extend(work);
                }

                // Added
                (Some(element_id), None) => {
                    let element = snapshot.get(element_id).unwrap();

                    let index =
                        if let Some(component_impl) = self.registry.get_by_id(element.type_id) {
                            let component = (component_impl.new)(element.props.as_ref());

                            assert_eq!(component.as_ref().type_id(), element.type_id);

                            self.tree.insert(Node {
                                component,
                                children: Vec::new(),
                            })
                        } else {
                            panic!("Unknown component ID {:?}", element.type_id);
                        };

                    match work_item.parent {
                        Some(parent_index) => {
                            let parent = self.tree.get_mut(parent_index).unwrap();
                            parent.children.push(index);
                        }

                        None => {
                            self.roots.push(index);
                        }
                    }

                    // Queue all of the element's children for addition.
                    let work = element.children.iter().copied().map(|id| WorkItem {
                        element: Some(id),
                        dom_node: None,
                        parent: Some(index),
                    });
                    queue.extend(work);
                }

                // Removed
                (None, Some(dom_index)) => {
                    let node = self.tree.remove(dom_index).unwrap();

                    // Queue all of this node's children for removal.
                    let work = node.children.into_iter().map(|index| WorkItem {
                        element: None,
                        dom_node: Some(index),
                        parent: Some(dom_index),
                    });
                    queue.extend(work);
                }

                (None, None) => unreachable!(),
            }
        }

        self.snapshot = Some(snapshot);
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
