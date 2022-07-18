use std::fmt;

use super::{Dom, DomInner};

impl fmt::Debug for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.try_borrow() {
            Ok(inner) => f
                .debug_struct("Dom")
                .field("root", &self.root)
                .field("nodes", &ViewTree(&inner))
                .finish(),

            Err(_) => f.debug_struct("Dom (Locked)").finish(),
        }
    }
}

struct ViewTree<'a>(&'a DomInner);

impl<'a> fmt::Debug for ViewTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dom = &self.0;
        let iter = dom.nodes.iter().map(|(id, node)| {
            format!("{id:?}: {:?}, children: {:?}", &node.widget, &node.children)
        });

        f.debug_list().entries(iter).finish()
    }
}
