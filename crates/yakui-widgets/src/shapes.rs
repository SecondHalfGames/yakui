use std::f32::consts::TAU;

use yakui_core::geometry::{Color, Rect, Vec2};
use yakui_core::paint::{PaintDom, PaintMesh, PaintRect, Vertex};
use yakui_core::TextureId;

use crate::auto_builders;
use crate::border::{Border, BorderRadius};

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

pub fn selection_halo(output: &mut PaintDom, rect: Rect, color: Color) {
    outline(output, rect, 2.0, color);
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

pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Color,
}

auto_builders!(Circle { color: Color });

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

pub struct RoundedRectangle {
    pub rect: Rect,
    pub color: Color,
    pub texture: Option<(TextureId, Rect)>,
    pub radius: BorderRadius,
    pub border: Option<Border>,
}

auto_builders!(RoundedRectangle {
    color: Color,
    texture: Option<(TextureId, Rect)>,
    border: Option<Border>,
});

impl RoundedRectangle {
    pub fn new<T: Into<BorderRadius>>(rect: Rect, radius: T) -> Self {
        Self {
            rect,
            color: Color::WHITE,
            texture: None,
            radius: radius.into(),
            border: None,
        }
    }

    pub fn add(&self, output: &mut PaintDom) {
        // Draw border background first if there's a border
        if let Some(border) = &self.border {
            self.draw_border(output, border);
        }

        let (rect, radius) = if let Some(border) = &self.border {
            let border_width = border.width;
            let inner_rect = Rect::from_pos_size(
                self.rect.pos() + Vec2::new(border_width, border_width),
                self.rect.size() - Vec2::new(border_width * 2.0, border_width * 2.0),
            );
            let inner_radius = BorderRadius {
                top_left: (self.radius.top_left - border_width).max(0.0),
                top_right: (self.radius.top_right - border_width).max(0.0),
                bottom_left: (self.radius.bottom_left - border_width).max(0.0),
                bottom_right: (self.radius.bottom_right - border_width).max(0.0),
            };
            (inner_rect, inner_radius)
        } else {
            (self.rect, self.radius)
        };

        let BorderRadius {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        } = radius;

        // We are not prepared to let a corner's radius be bigger than a side's
        // half-length.
        let max_radius = (rect.size().x / 2.0).min(rect.size().y / 2.0);

        let top_left = top_left.min(max_radius);
        let top_right = top_right.min(max_radius);
        let bottom_left = bottom_left.min(max_radius);
        let bottom_right = bottom_right.min(max_radius);

        // Fallback to a rectangle if the radius is too small.
        if top_left < 1.0 && top_right < 1.0 && bottom_left < 1.0 && bottom_right < 1.0 {
            let mut p = PaintRect::new(rect);
            p.texture = self.texture;
            p.color = self.color;
            return p.add(output);
        }

        let color = self.color.to_linear();

        let max_radius = top_left.max(top_right).max(bottom_left).max(bottom_right);
        let slices = f32::ceil(TAU / 8.0 / f32::acos(1.0 - 0.2 / max_radius)) as u32;

        // 3 rectangles and 4 corners
        let mut vertices = Vec::with_capacity(4 * 3 + (slices + 2) as usize * 4);
        let mut indices = Vec::with_capacity(6 * 3 + slices as usize * (3 * 4));

        let (uv_offset, uv_factor) = self
            .texture
            .map(|(_, texture_rect)| (texture_rect.pos(), texture_rect.size() / rect.size()))
            .unwrap_or((Vec2::ZERO, Vec2::ZERO));

        let calc_uv = |position| {
            if self.texture.is_none() {
                return Vec2::ZERO;
            }
            (position - rect.pos()) * uv_factor + uv_offset
        };

        let create_vertex = |pos| Vertex::new(pos, calc_uv(pos), color);

        let mut rectangle = |min: Vec2, max: Vec2| {
            let base_vertex = vertices.len();

            let size = max - min;
            let rect_vertices = RECT_POS
                .map(Vec2::from)
                .map(|vert| create_vertex(vert * size + min));

            let rect_indices = RECT_INDEX.map(|index| index + base_vertex as u16);

            vertices.extend(rect_vertices);
            indices.extend(rect_indices);
        };

        // Top
        rectangle(
            Vec2::new(rect.pos().x + top_left, rect.pos().y),
            Vec2::new(
                rect.max().x - top_right,
                rect.pos().y + top_left.max(top_right),
            ),
        );

        // Left
        rectangle(
            Vec2::new(rect.pos().x, rect.pos().y + top_left),
            Vec2::new(
                rect.pos().x + bottom_left.max(top_left),
                rect.max().y - bottom_left,
            ),
        );

        // Center
        rectangle(
            Vec2::new(
                rect.pos().x + bottom_left.max(top_left),
                rect.pos().y + top_left.max(top_right),
            ),
            Vec2::new(
                rect.max().x - bottom_right.max(top_right),
                rect.max().y - bottom_left.max(bottom_right),
            ),
        );

        // Right
        rectangle(
            Vec2::new(
                rect.max().x - bottom_right.max(top_right),
                rect.pos().y + top_right,
            ),
            Vec2::new(rect.max().x, rect.max().y - bottom_right),
        );

        // Bottom
        rectangle(
            Vec2::new(
                rect.pos().x + bottom_left,
                rect.max().y - bottom_left.max(bottom_right),
            ),
            Vec2::new(rect.max().x - bottom_right, rect.max().y),
        );

        let mut corner = |center: Vec2, radius: f32, start_angle: f32| {
            if radius < 1.0 {
                return;
            }

            let center_vertex = vertices.len();
            vertices.push(create_vertex(center));

            let first_offset = radius * Vec2::new(start_angle.cos(), -start_angle.sin());
            vertices.push(create_vertex(center + first_offset));

            for i in 1..=slices {
                let percent = i as f32 / slices as f32;
                let angle = start_angle + percent * TAU / 4.0;
                let offset = radius * Vec2::new(angle.cos(), -angle.sin());
                let index = vertices.len();
                vertices.push(create_vertex(center + offset));

                indices.extend_from_slice(&[
                    center_vertex as u16,
                    (index - 1) as u16,
                    index as u16,
                ]);
            }
        };

        corner(
            Vec2::new(rect.max().x - top_right, rect.pos().y + top_right),
            top_right,
            0.0,
        );
        corner(
            Vec2::new(rect.pos().x + top_left, rect.pos().y + top_left),
            top_left,
            TAU / 4.0,
        );
        corner(
            Vec2::new(rect.pos().x + bottom_left, rect.max().y - bottom_left),
            bottom_left,
            TAU / 2.0,
        );
        corner(
            Vec2::new(rect.max().x - bottom_right, rect.max().y - bottom_right),
            bottom_right,
            3.0 * TAU / 4.0,
        );

        if let Some(border) = &self.border {
            self.draw_border(output, border);
        }

        let mut mesh = PaintMesh::new(vertices, indices);
        mesh.texture = self.texture;
        output.add_mesh(mesh);
    }

    // Just draws a larger rectangle behind the main one... probably has issues with opacity?
    fn draw_border(&self, output: &mut PaintDom, border: &Border) {
        let mut border_shape = RoundedRectangle::new(self.rect, self.radius);
        border_shape.color = border.color;
        border_shape.add(output);
    }
}
