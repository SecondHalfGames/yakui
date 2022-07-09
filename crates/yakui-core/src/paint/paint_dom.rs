use thunderdome::Arena;
use thunderdome::Index;

use crate::dom::Dom;
use crate::layout::LayoutDom;

use super::Output;
use super::Texture;

#[derive(Debug)]
pub struct PaintDom {
    textures: Arena<Texture>,
    textures_generation: Arena<u8>,
}

impl PaintDom {
    pub fn new() -> Self {
        Self {
            textures: Arena::new(),
            textures_generation: Arena::new(),
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

    pub fn create_texture(&mut self, texture: Texture) -> Index {
        let index = self.textures.insert(texture);
        self.textures_generation.insert_at(index, 0);
        index
    }

    pub fn remove_texture(&mut self, index: Index) {
        self.textures.remove(index);
        self.textures_generation.remove(index);
    }

    pub fn get_texture(&self, index: Index) -> Option<&Texture> {
        self.textures.get(index)
    }

    pub fn modify_texture(&mut self, index: Index) -> Option<&mut Texture> {
        let texture = self.textures.get_mut(index)?;
        let generation = self.textures_generation.get_mut(index)?;
        *generation = generation.wrapping_add(1);
        Some(texture)
    }
}
