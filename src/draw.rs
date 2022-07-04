use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

pub struct Output {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec2,
    pub texcoord: Vec2,
    pub color: Vec4,
}
