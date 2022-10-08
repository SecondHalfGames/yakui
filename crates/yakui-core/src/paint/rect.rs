use glam::Vec2;

use crate::geometry::{Color, Rect};
use crate::TextureId;

use super::{PaintDom, PaintMesh, Pipeline, Vertex};

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
#[allow(missing_docs)]
pub struct PaintRect {
    pub rect: Rect,
    pub color: Color,
    pub texture: Option<(TextureId, Rect)>,
    pub pipeline: Pipeline,
}

impl PaintRect {
    /// Create a new `PaintRect` with the default pipeline, no texture, and the
    /// given rectangle.
    ///
    /// (0, 0) is the top-left corner of the screen, while (1, 1) is the
    /// bottom-right corner of the screen. Widgets must take the viewport into
    /// account.
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            color: Color::WHITE,
            texture: None,
            pipeline: Pipeline::Main,
        }
    }

    /// Add this rectangle to the PaintDom to be drawn this frame.
    pub fn add(&self, output: &mut PaintDom) {
        let size = self.rect.size();
        let pos = self.rect.pos();
        let color = self.color.to_linear();
        let texture_rect = match self.texture {
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

        let mut mesh = PaintMesh::new(vertices, RECT_INDEX);
        mesh.texture = self.texture;
        mesh.pipeline = self.pipeline;

        output.add_mesh(mesh);
    }
}
