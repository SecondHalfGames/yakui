use std::f32::consts::TAU;

use yakui_core::geometry::{Color, Rect, Vec2};
use yakui_core::paint::{PaintDom, PaintMesh, Vertex};

pub fn cross(output: &mut PaintDom, rect: Rect, color: Color) {
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

    let color = color.to_linear();

    let vertices = POSITIONS
        .into_iter()
        .map(Vec2::from)
        .map(|pos| Vertex::new(rect.pos() + pos * rect.size(), [0.0, 0.0], color));

    let mesh = PaintMesh::new(vertices, INDICES);
    output.add_mesh(mesh);
}

pub fn selection_halo(output: &mut PaintDom, rect: Rect) {
    outline(output, rect, 2.0, Color::WHITE);
}

pub fn outline(output: &mut PaintDom, rect: Rect, w: f32, color: Color) {
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

    let color = color.to_linear();

    let vertices = positions
        .into_iter()
        .map(Vec2::from)
        .map(|pos| Vertex::new(rect.pos() + pos, [0.0, 0.0], color));

    let mesh = PaintMesh::new(vertices, INDICES);
    output.add_mesh(mesh);
}

#[non_exhaustive]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            color: Color::WHITE,
        }
    }

    pub fn add(&self, output: &mut PaintDom) {
        let color = self.color.to_linear();
        let mut vertices = Vec::new();
        let segments = f32::ceil(TAU / 2.0 / f32::acos(1.0 - 0.2 / self.radius));

        for i in 0..segments as u32 {
            let angle = TAU * (i as f32) / segments;
            let (y, x) = angle.sin_cos();
            let pos = self.center + Vec2::new(x, y) * self.radius;

            vertices.push(Vertex::new(pos, [0.0, 0.0], color));
        }

        vertices.push(Vertex::new(self.center, [0.0, 0.0], color));
        let middle_vertex = (vertices.len() - 1) as u16;

        let mut indices = Vec::new();
        let segments = segments as i16;

        for i in 0i16..segments {
            indices.push(i as u16);
            indices.push((i - 1).rem_euclid(segments) as u16);
            indices.push(middle_vertex);
        }

        let mesh = PaintMesh::new(vertices, indices);
        output.add_mesh(mesh);
    }
}

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
pub struct RoundedRectangle {
    pub rect: Rect,
    pub radius: f32,
    pub color: Color,
}

impl RoundedRectangle {
    pub fn new(rect: Rect, radius: f32) -> Self {
        Self {
            rect,
            radius,
            color: Color::WHITE,
        }
    }

    pub fn add(&self, output: &mut PaintDom) {
        let rect = self.rect;
        let color = self.color.to_linear();

        // We are not prepared to let a corner's radius be bigger than a side's
        // half-length.
        let radius = self
            .radius
            .min(rect.size().x / 2.0)
            .min(rect.size().y / 2.0);

        let slices = if radius >= 1.0 {
            f32::ceil(TAU / 8.0 / f32::acos(1.0 - 0.2 / radius)) as u32
        } else {
            1
        };

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut rectangle = |min: Vec2, max: Vec2| {
            let base_vertex = vertices.len();

            let size = max - min;
            let rect_vertices = RECT_POS
                .map(Vec2::from)
                .map(|vert| Vertex::new(vert * size + min, Vec2::ZERO, color));

            let rect_indices = RECT_INDEX.map(|index| index + base_vertex as u16);

            vertices.extend(rect_vertices);
            indices.extend(rect_indices);
        };

        rectangle(
            Vec2::new(rect.pos().x + radius, rect.pos().y),
            Vec2::new(rect.max().x - radius, rect.pos().y + radius),
        );
        rectangle(
            Vec2::new(rect.pos().x, rect.pos().y + radius),
            Vec2::new(rect.max().x, rect.max().y - radius),
        );
        rectangle(
            Vec2::new(rect.pos().x + radius, rect.max().y - radius),
            Vec2::new(rect.max().x - radius, rect.max().y),
        );

        let mut corner = |center: Vec2, start_angle: f32| {
            let center_vertex = vertices.len();
            vertices.push(Vertex::new(center, Vec2::ZERO, color));

            let first_offset = radius * Vec2::new(start_angle.cos(), -start_angle.sin());
            vertices.push(Vertex::new(center + first_offset, Vec2::ZERO, color));

            for i in 1..=slices {
                let percent = i as f32 / slices as f32;
                let angle = start_angle + percent * TAU / 4.0;
                let offset = radius * Vec2::new(angle.cos(), -angle.sin());
                let index = vertices.len();
                vertices.push(Vertex::new(center + offset, Vec2::ZERO, color));

                indices.extend_from_slice(&[
                    center_vertex as u16,
                    (index - 1) as u16,
                    index as u16,
                ]);
            }
        };

        corner(Vec2::new(rect.max().x - radius, rect.pos().y + radius), 0.0);
        corner(
            Vec2::new(rect.pos().x + radius, rect.pos().y + radius),
            TAU / 4.0,
        );
        corner(
            Vec2::new(rect.pos().x + radius, rect.max().y - radius),
            TAU / 2.0,
        );
        corner(
            Vec2::new(rect.max().x - radius, rect.max().y - radius),
            3.0 * TAU / 4.0,
        );

        let mesh = PaintMesh::new(vertices, indices);
        output.add_mesh(mesh);
    }
}
