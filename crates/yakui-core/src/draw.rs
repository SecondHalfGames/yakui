use glam::{Vec2, Vec4};

use crate::dom::{Dom, LayoutDom};

#[non_exhaustive]
pub struct Output {
    pub meshes: Vec<Mesh>,
}

impl Output {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub(crate) fn draw(dom: &Dom, layout: &LayoutDom) -> Output {
        let mut output = Output::new();

        for &node_index in dom.roots() {
            let node = dom.get(node_index).unwrap();
            node.component.draw(dom, layout, &mut output);
        }

        output
    }
}

#[non_exhaustive]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Vertex {
    pub position: Vec2,
    pub texcoord: Vec2,
    pub color: Vec4,
}

impl Vertex {
    pub fn new<P, T, C>(position: P, texcoord: T, color: C) -> Self
    where
        P: Into<Vec2>,
        T: Into<Vec2>,
        C: Into<Vec4>,
    {
        Self {
            position: position.into(),
            texcoord: texcoord.into(),
            color: color.into(),
        }
    }
}
