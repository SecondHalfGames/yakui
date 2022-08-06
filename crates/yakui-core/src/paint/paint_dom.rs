use glam::Vec2;

use crate::dom::Dom;
use crate::geometry::Rect;
use crate::id::{TextureId, WidgetId};
use crate::layout::LayoutDom;

use super::primitives::{PaintCall, PaintMesh, PaintRect, Vertex};
use super::texture::Texture;

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

/// Contains all information about how to paint the current set of widgets.
#[derive(Debug)]
pub struct PaintDom {
    texture_edits: Vec<TextureEdit>,
    calls: Vec<PaintCall>,
    viewport: Rect,
}

impl PaintDom {
    /// Create a new, empty Paint DOM.
    pub fn new() -> Self {
        Self {
            texture_edits: Vec::new(),
            calls: Vec::new(),
            viewport: Rect::ONE,
        }
    }

    /// Just clears the texture edits, which should have been consumed last frame
    pub(crate) fn start(&mut self) {
        self.texture_edits.clear();
    }

    pub(crate) fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    /// Paint a specific widget. This function is usually called as part of an
    /// implementation of [`Widget::paint`][crate::widget::Widget::paint].
    ///
    /// Must only be called once per widget per paint pass.
    pub fn paint(&mut self, dom: &Dom, layout: &LayoutDom, id: WidgetId) {
        profiling::scope!("PaintDom::paint");

        let node = dom.get(id).unwrap();
        dom.enter(id);
        node.widget.paint(dom, layout, self);
        dom.exit(id);
    }

    /// Paint all of the widgets in the given DOM.
    pub fn paint_all(&mut self, dom: &Dom, layout: &LayoutDom) {
        profiling::scope!("PaintDom::paint_all");
        log::debug!("PaintDom:paint_all()");

        self.calls.clear();

        let node = dom.get(dom.root()).unwrap();
        node.widget.paint(dom, layout, self);
    }

    /// Returns a mutable handle to a texture given its ID.
    ///
    /// The texture will be marked as dirty, which may cause it to be reuploaded
    /// to the GPU by the renderer.
    pub fn modify_texture(&mut self, id: TextureId, texture: Texture) {
        self.texture_edits.push(TextureEdit::Modify(id, texture));
    }

    /// Takes all the texture edits. These must be consumed *before* processing the [PaintCall]s for
    /// this frame.
    pub fn texture_edits(&self) -> &[TextureEdit] {
        &self.texture_edits
    }

    /// Returns a list of paint calls that could be used to draw the UI.
    pub fn calls(&self) -> &[PaintCall] {
        self.calls.as_slice()
    }

    /// Add a mesh to be painted.
    pub fn add_mesh<V, I>(&mut self, mesh: PaintMesh<V, I>)
    where
        V: IntoIterator<Item = Vertex>,
        I: IntoIterator<Item = u16>,
    {
        profiling::scope!("PaintDom::add_mesh");

        let texture_id = mesh.texture.map(|(index, _rect)| index);

        let call = match self.calls.last_mut() {
            Some(call) if call.texture == texture_id && call.pipeline == mesh.pipeline => call,
            _ => {
                let mut new_mesh = PaintCall::new();
                new_mesh.texture = texture_id;
                new_mesh.pipeline = mesh.pipeline;

                self.calls.push(new_mesh);
                self.calls.last_mut().unwrap()
            }
        };

        let indices = mesh
            .indices
            .into_iter()
            .map(|index| index + call.vertices.len() as u16);
        call.indices.extend(indices);

        let vertices = mesh.vertices.into_iter().map(|mut vertex| {
            vertex.position = (vertex.position + self.viewport.pos()) / self.viewport.size();
            vertex
        });
        call.vertices.extend(vertices);
    }

    /// Add a rectangle to be painted. This is a convenience function over
    /// [`PaintDom::add_mesh`].
    pub fn add_rect(&mut self, rect: PaintRect) {
        let size = rect.rect.size();
        let pos = rect.rect.pos();
        let color = rect.color.to_linear();
        let texture_rect = match rect.texture {
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
        mesh.texture = rect.texture;
        mesh.pipeline = rect.pipeline;

        self.add_mesh(mesh);
    }
}

/// An edit to a texture id. Clients must consume these edits as appropriate.
#[derive(Debug)]
pub enum TextureEdit {
    // how does *add* work?
    /// This texture id has been modified.
    Modify(TextureId, Texture),
    /// This texture has been removed.
    Remove(TextureId),
}
