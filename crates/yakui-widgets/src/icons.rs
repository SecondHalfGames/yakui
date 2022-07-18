use std::f32::consts::TAU;

use yakui_core::geometry::{Color3, Rect, Vec2};
use yakui_core::paint::{PaintDom, PaintMesh, Vertex};

pub fn cross(output: &mut PaintDom, rect: Rect, color: Color3) {
    static POSITIONS: [[f32; 2]; 12] = [
        // Top
        [0.25, 0.0], // 0
        [0.75, 0.0], // 1
        // Left
        [0.0, 0.25], // 2
        [0.0, 0.75], // 3
        // Bottom
        [0.25, 1.0], // 4
        [0.75, 1.0], // 5
        // Right
        [1.0, 0.25], // 6
        [1.0, 0.75], // 7
        // Inner points
        [0.5, 0.25], // 8
        [0.25, 0.5], // 9
        [0.5, 0.75], // 10
        [0.75, 0.5], // 11
    ];

    #[rustfmt::skip]
    static INDICES: [u16; 18] = [
        // '\' part of the X
        0, 2, 5,
        5, 7, 0,
        // bottom left square
        9, 3, 4,
        4, 10, 9,
        // top right square
        1, 8, 11,
        11, 6, 1,
    ];

    let color = color.to_linear().extend(1.0);

    let vertices = POSITIONS
        .into_iter()
        .map(Vec2::from)
        .map(|pos| Vertex::new(rect.pos() + pos * rect.size(), [0.0, 0.0], color));

    let mesh = PaintMesh::new(vertices, INDICES);
    output.add_mesh(mesh);
}

pub fn selection_halo(output: &mut PaintDom, rect: Rect) {
    outline(output, rect, 2.0, Color3::WHITE);
}

pub fn outline(output: &mut PaintDom, rect: Rect, w: f32, color: Color3) {
    let vw = rect.size().x;
    let vh = rect.size().y;

    #[rustfmt::skip]
    let positions: [[f32; 2]; 12] = [
        // Main rectangle
        [0.0, 0.0], // 0
        [0.0, vh], // 1
        [ vw, vh], // 2
        [ vw, 0.0], // 3
        // Left border
        [0.0,      w], // 4
        [0.0, vh - w], // 5
        [  w, vh - w], // 6
        [  w,      w], // 7
        // Right border
        [vw - w,      w], // 8
        [vw - w, vh - w], // 9
        [    vw, vh - w], // 10
        [    vw,      w], // 11
    ];

    #[rustfmt::skip]
    static INDICES: [u16; 24] = [
        // Top
        0, 4, 11,
        11, 3, 0,
        // Bottom
        5, 1, 2,
        2, 10, 5,
        // Left
        4, 5, 6,
        6, 7, 4,
        // Right
        8, 9, 10,
        10, 11, 8,
    ];

    let color = color.to_linear().extend(1.0);

    let vertices = positions
        .into_iter()
        .map(Vec2::from)
        .map(|pos| Vertex::new(rect.pos() + pos, [0.0, 0.0], color));

    let mesh = PaintMesh::new(vertices, INDICES);
    output.add_mesh(mesh);
}

pub struct PaintCircle {
    pub center: Vec2,
    pub radius: f32,
    pub segments: u16,
    pub color: Color3,
}

impl PaintCircle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            segments: 24,
            color: Color3::WHITE,
        }
    }

    pub fn add(&self, output: &mut PaintDom) {
        let color = self.color.to_linear().extend(1.0);
        let mut vertices = Vec::new();
        let segments = self.segments as f32;

        for i in 0..self.segments {
            let angle = TAU * (i as f32) / segments;
            let (y, x) = angle.sin_cos();
            let pos = self.center + Vec2::new(x, y) * self.radius;

            vertices.push(Vertex::new(pos, [0.0, 0.0], color));
        }

        vertices.push(Vertex::new(self.center, [0.0, 0.0], color));
        let middle_vertex = (vertices.len() - 1) as u16;

        let mut indices = Vec::new();
        let segments = self.segments as i16;

        for i in 0i16..segments {
            indices.push(i as u16);
            indices.push((i - 1).rem_euclid(segments) as u16);
            indices.push(middle_vertex);
        }

        let mesh = PaintMesh::new(vertices, indices);
        output.add_mesh(mesh);
    }
}
