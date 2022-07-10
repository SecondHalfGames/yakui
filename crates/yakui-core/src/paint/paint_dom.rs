use glam::Vec2;
use thunderdome::Arena;
use thunderdome::Index;

use crate::dom::Dom;
use crate::layout::LayoutDom;
use crate::Rect;

use super::Mesh;
use super::PaintRect;
use super::Texture;
use super::Vertex;

#[rustfmt::skip]
const RECT_POS: [[f32; 2]; 4] = [
    [0.0, 0.0],
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0]
];

#[rustfmt::skip]
const RECT_INDEX: [u16; 6] = [
    0, 1, 2,
    3, 0, 2,
];

#[derive(Debug)]
pub struct PaintDom {
    textures: Arena<Texture>,
    meshes: Vec<Mesh>,
}

impl PaintDom {
    pub fn new() -> Self {
        Self {
            textures: Arena::new(),
            meshes: Vec::new(),
        }
    }

    pub fn paint(&mut self, dom: &Dom, layout: &LayoutDom, index: Index) {
        let node = dom.get(index).unwrap();
        dom.enter(index);
        node.widget.paint(dom, layout, self);
        dom.exit(index);
    }

    pub fn paint_all(&mut self, dom: &Dom, layout: &LayoutDom) {
        self.meshes.clear();

        let node = dom.get(dom.root()).unwrap();
        node.widget.paint(dom, layout, self);
    }

    pub fn create_texture(&mut self, texture: Texture) -> Index {
        self.textures.insert(texture)
    }

    pub fn remove_texture(&mut self, index: Index) {
        self.textures.remove(index);
    }

    pub fn get_texture(&self, index: Index) -> Option<&Texture> {
        self.textures.get(index)
    }

    pub fn modify_texture(&mut self, index: Index) -> Option<&mut Texture> {
        let texture = self.textures.get_mut(index)?;
        texture.generation = texture.generation.wrapping_add(1);
        Some(texture)
    }

    pub fn textures(&self) -> impl Iterator<Item = (Index, &Texture)> {
        self.textures.iter()
    }

    pub fn meshes(&self) -> &[Mesh] {
        self.meshes.as_slice()
    }

    pub fn add_rect(&mut self, rect: PaintRect) {
        let size = rect.rect.size();
        let pos = rect.rect.pos();
        let color = rect.color.as_vec4(1.0);

        let texture_id = rect.texture.map(|(index, _rect)| index);
        let mesh = match self.meshes.last_mut() {
            Some(mesh) if mesh.texture == texture_id && mesh.pipeline == rect.pipeline => mesh,
            _ => {
                let mut new_mesh = Mesh::new();
                new_mesh.texture = texture_id;
                new_mesh.pipeline = rect.pipeline;

                self.meshes.push(new_mesh);
                self.meshes.last_mut().unwrap()
            }
        };

        let texture_rect = match rect.texture {
            Some((_index, rect)) => rect,
            None => Rect::from_pos_size(Vec2::ZERO, Vec2::ONE),
        };

        let vertices = RECT_POS.map(Vec2::from).map(|vert| {
            Vertex::new(
                vert * size + pos,
                vert * texture_rect.size() + texture_rect.pos(),
                color,
            )
        });

        let indices = RECT_INDEX.map(|index| index + mesh.vertices.len() as u16);

        mesh.vertices.extend(vertices);
        mesh.indices.extend(indices);
    }
}
