use glam::Vec2;

use crate::geometry::{Color3, Rect};

use super::{Mesh, PaintRect, Vertex};

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

#[non_exhaustive]
pub struct Output {
    pub meshes: Vec<Mesh>,
}

impl Output {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn add_rect(&mut self, rect: PaintRect) {
        let size = rect.rect.size();
        let pos = rect.rect.pos();
        let color = rect.color.as_vec4(1.0);

        let mesh = match self.meshes.last_mut() {
            Some(mesh) if mesh.texture == rect.texture => mesh,
            _ => {
                let mut new_mesh = Mesh::new();
                new_mesh.texture = rect.texture;

                self.meshes.push(new_mesh);
                self.meshes.last_mut().unwrap()
            }
        };

        let vertices = RECT_POS
            .map(Vec2::from)
            .map(|vert| Vertex::new(vert * size + pos, vert, color));

        let indices = RECT_INDEX.map(|index| index + mesh.vertices.len() as u16);

        mesh.vertices.extend(vertices);
        mesh.indices.extend(indices);
    }

    pub fn rect(&mut self, pos: Vec2, size: Vec2, color: Color3) {
        self.add_rect(PaintRect {
            rect: Rect::from_pos_size(pos, size),
            color,
            texture: None,
        });
    }
}
