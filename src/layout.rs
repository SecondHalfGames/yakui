use glam::{Vec2, Vec4};

pub struct Layout {
    pub meshes: Vec<Mesh>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture: (),
}

pub struct Vertex {
    pub position: Vec2,
    pub texcoord: Vec2,
    pub color: Vec4,
}

#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min: Option<Vec2>,
    pub max: Option<Vec2>,
}

impl Constraints {
    pub fn constrain(&self, mut base: Vec2) -> Vec2 {
        if let Some(min) = self.min {
            base = base.max(min);
        }

        if let Some(max) = self.max {
            base = base.min(max);
        }

        base
    }
}
