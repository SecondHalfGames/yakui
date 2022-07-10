use std::fmt;

use super::Dom;

impl fmt::Debug for Dom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dom")
            .field("roots", &self.roots)
            .field("tree", &ViewTree(self))
            .finish()
    }
}

struct ViewTree<'a>(&'a Dom);

impl<'a> fmt::Debug for ViewTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dom = &self.0;
        let iter = dom.nodes.iter().map(|(index, node)| {
            let debug = node.widget.as_debug();

            let children: Vec<_> = node.children.iter().map(|index| index.slot()).collect();

            format!("{}: {debug:?}, children: {:?}", index.slot(), children)
        });

        f.debug_list().entries(iter).finish()
    }
}
