use glam::{Vec2, Vec4};

use crate::geometry::Rect;
use crate::id::TextureId;

#[allow(missing_docs)]
pub struct PaintMesh<V, I> {
    /// Vertex positions, the unit here is in logical pixels.
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

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct YakuiPaintCall {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture: Option<TextureId>,
    pub pipeline: Pipeline,
}

impl YakuiPaintCall {
    /// Create a new empty `YakuiPaintCall`.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            texture: None,
            pipeline: Pipeline::Main,
        }
    }
}

/// A user-managed ID for an externally managed paint call
pub type UserPaintCallId = u64;

/// Represents a paint call that may be internal to yakui or handled by the user.
#[derive(Debug)]
pub enum PaintCall {
    /// A paint call that is internal to yakui.
    Internal(YakuiPaintCall),

    /// A paint call that is managed by the user or renderer.
    User(UserPaintCallId),
}

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
#[repr(C)]
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
pub enum Pipeline {
    /// Pipeline for drawing most geometry: vertices and an optional color
    /// texture.
    #[default]
    Main,

    /// Pipeline for drawing text: vertices and a coverage glyph texture.
    Text,
}
