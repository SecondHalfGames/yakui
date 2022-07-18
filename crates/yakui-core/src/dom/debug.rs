use std::fmt;

use thunderdome::Arena;

use super::{Dom, DomNode};

impl fmt::Debug for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.nodes.try_borrow() {
            Ok(nodes) => f
                .debug_struct("Dom")
                .field("root", &self.inner.root)
                .field("nodes", &ViewTree(&nodes))
                .finish(),

            Err(_) => f.debug_struct("Dom (Locked)").finish(),
        }
    }
}

struct ViewTree<'a>(&'a Arena<DomNode>);

impl<'a> fmt::Debug for ViewTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = &self.0;
        let iter = nodes.iter().map(|(id, node)| {
            format!("{id:?}: {:?}, children: {:?}", &node.widget, &node.children)
        });

        f.debug_list().entries(iter).finish()
    }
}
