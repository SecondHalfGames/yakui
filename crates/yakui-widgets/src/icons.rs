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
