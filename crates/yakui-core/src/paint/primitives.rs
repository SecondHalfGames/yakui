use glam::{Vec2, Vec4};

use crate::geometry::{Color3, Rect};
use crate::id::TextureId;

#[non_exhaustive]
#[allow(missing_docs)]
pub struct PaintRect {
    pub rect: Rect,
    pub color: Color3,
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
            color: Color3::WHITE,
            texture: None,
            pipeline: Pipeline::Main,
        }
    }
}

#[non_exhaustive]
#[allow(missing_docs)]
pub struct PaintMesh<V, I> {
    pub vertices: V,
    pub indices: I,
    pub texture: Option<(TextureId, Rect)>,
    pub pipeline: Pipeline,
}

impl<V, I> PaintMesh<V, I>
where
    V: IntoIterator<Item = Vertex>,
    I: IntoIterator<Item = u16>,
{
    /// Create a new `PaintMesh` with the default pipeline, no texture, and the
    /// given vertices and indices.
    pub fn new(vertices: V, indices: I) -> Self {
        Self {
            vertices,
            indices,
            texture: None,
            pipeline: Pipeline::Main,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct PaintCall {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture: Option<TextureId>,
    pub pipeline: Pipeline,
}

impl PaintCall {
    /// Create a new empty `PaintCall`.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            texture: None,
            pipeline: Pipeline::Main,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
#[allow(missing_docs)]
pub struct Vertex {
    pub position: Vec2,
    pub texcoord: Vec2,
    pub color: Vec4,
}

impl Vertex {
    /// Create a new `Vertex`.
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

/// The graphics pipeline that a draw call should be executed with.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Pipeline {
    /// Pipline for drawing most geometry: vertices and an optional color
    /// texture.
    #[default]
    Main,

    /// Pipeline for drawing text: vertices and a coverage glyph texture.
    Text,
}
