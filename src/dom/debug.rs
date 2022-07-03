use std::fmt;

use super::Dom;

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
