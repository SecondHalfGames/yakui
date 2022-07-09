use thunderdome::Arena;

use crate::dom::Dom;
use crate::layout::LayoutDom;

use super::Output;
use super::Texture;

#[derive(Debug)]
pub struct PaintDom {
    textures: Arena<Texture>,
}

impl PaintDom {
    pub fn new() -> Self {
        Self {
            textures: Arena::new(),
        }
    }

    pub fn paint(&self, dom: &Dom, layout: &LayoutDom) -> Output {
        let mut output = Output::new();

        for &node_index in dom.roots() {
            let node = dom.get(node_index).unwrap();
            node.component.paint(dom, layout, &mut output);
        }

        output
    }
}
