use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use glam::Vec2;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use thunderdome::Arena;

use crate::dom::Dom;
use crate::geometry::Rect;
use crate::id::{ManagedTextureId, WidgetId};
use crate::layout::LayoutDom;
use crate::paint::{PaintCall, Pipeline};
use crate::widget::PaintContext;

use super::layers::PaintLayers;
use super::primitives::{PaintMesh, Vertex};
use super::texture::{Texture, TextureChange};

#[derive(Debug, Clone, Copy, Default)]
/// Contains all information about the limits of the paint device.
pub struct PaintLimits {
    /// Maximum texture size of a 1D texture.
    pub max_texture_size_1d: u32,
    /// Maximum texture size of a 2D texture.
    pub max_texture_size_2d: u32,
    /// Maximum texture size of a 3D texture.
    pub max_texture_size_3d: u32,
}

/// Contains all information about how to paint the current set of widgets.
#[derive(Debug)]
pub struct PaintDom {
    textures: Arc<RwLock<Textures>>,
    surface_size: Vec2,
    unscaled_viewport: Rect,
    scale_factor: f32,
    limits: Option<PaintLimits>,

    layers: PaintLayers,
    clip_stack: Vec<Rect>,
}

/// Stores textures for one or more `PaintDom` instances.
#[derive(Debug, Default)]
pub struct Textures {
    storage: Arena<Texture>,
    texture_edits: HashMap<ManagedTextureId, TextureChange>,
}

impl Textures {
    /// Add a texture to the Paint DOM, returning an ID that can be used to
    /// reference it later.
    pub fn add(&mut self, texture: Texture) -> ManagedTextureId {
        let texture = self.storage.insert(texture);

        let id = ManagedTextureId::new(texture);
        self.texture_edits.insert(id, TextureChange::Added);
        id
    }

    /// Remove a texture from the Paint DOM.
    pub fn remove(&mut self, id: ManagedTextureId) {
        self.storage.remove(id.index());
        self.texture_edits.insert(id, TextureChange::Removed);
    }

    /// Retrieve a texture by its ID, if it exists.
    pub fn get(&self, id: ManagedTextureId) -> Option<&Texture> {
        self.storage.get(id.index())
    }

    /// Retrieves a mutable reference to a texture by its ID.
    pub fn get_mut(&mut self, id: ManagedTextureId) -> Option<&mut Texture> {
        self.storage.get_mut(id.index())
    }

    /// Mark a texture as modified so that changes can be detected.
    pub fn mark_modified(&mut self, id: ManagedTextureId) {
        self.texture_edits.insert(id, TextureChange::Modified);
    }

    /// Returns an iterator over all textures known to the Paint DOM.
    pub fn iter(&self) -> impl Iterator<Item = (ManagedTextureId, &Texture)> {
        self.storage
            .iter()
            .map(|(index, texture)| (ManagedTextureId::new(index), texture))
    }

    /// Iterates over the list of changes that happened to yakui-managed
    /// textures this frame.
    ///
    /// This is useful for renderers that need to upload or remove GPU resources
    /// related to textures.
    pub fn edits(&self) -> impl Iterator<Item = (ManagedTextureId, TextureChange)> + '_ {
        self.texture_edits.iter().map(|(&id, &edit)| (id, edit))
    }
}

impl PaintDom {
    /// Create a new, empty Paint DOM.
    pub fn new() -> Self {
        Self {
            textures: Default::default(),
            surface_size: Vec2::ONE,
            unscaled_viewport: Rect::ONE,
            scale_factor: 1.0,
            limits: None,

            layers: PaintLayers::new(),
            clip_stack: Vec::new(),
        }
    }

    /// Create a new `PaintDom` that shares resources with this existing one.
    pub fn fork(&self) -> Self {
        Self {
            textures: self.textures.clone(),
            ..Self::new()
        }
    }

    /// Gets the paint limits.
    pub fn limits(&self) -> Option<PaintLimits> {
        self.limits
    }

    /// Sets the paint limits, should be called once by rendering backends.
    pub fn set_limit(&mut self, limits: PaintLimits) {
        self.limits = Some(limits);
    }

    /// Prepares the PaintDom to be updated for the frame.
    pub fn start(&mut self) {
        self.textures_mut().texture_edits.clear();
        self.clip_stack.clear();
    }

    /// Returns the size of the surface that is being painted onto.
    pub fn surface_size(&self) -> Vec2 {
        self.surface_size
    }

    /// Set the size of the surface that yakui is being rendered on.
    pub(crate) fn set_surface_size(&mut self, size: Vec2) {
        self.surface_size = size;
    }

    pub(crate) fn set_unscaled_viewport(&mut self, viewport: Rect) {
        self.unscaled_viewport = viewport;
    }

    pub(crate) fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    /// Paint a specific widget. This function is usually called as part of an
    /// implementation of [`Widget::paint`][crate::widget::Widget::paint].
    ///
    /// Must only be called once per widget per paint pass.
    pub fn paint(&mut self, dom: &Dom, layout: &LayoutDom, id: WidgetId) {
        profiling::scope!("PaintDom::paint");

        let layout_node = layout.get(id).unwrap();
        if layout_node.clipping_enabled {
            self.push_clip(layout_node.rect);
        }
        if layout_node.new_layer {
            self.layers.push();
        }

        dom.enter(id);

        let context = PaintContext {
            dom,
            layout,
            paint: self,
        };
        let node = dom.get(id).unwrap();
        node.widget.paint(context);

        dom.exit(id);

        if layout_node.clipping_enabled {
            self.pop_clip();
        }
        if layout_node.new_layer {
            self.layers.pop();
        }
    }

    /// Paint all of the widgets in the given DOM.
    pub fn paint_all(&mut self, dom: &Dom, layout: &LayoutDom) {
        profiling::scope!("PaintDom::paint_all");
        log::debug!("PaintDom:paint_all()");

        self.layers.clear();
        self.paint(dom, layout, dom.root());
    }

    /// Add a texture to the Paint DOM, returning an ID that can be used to
    /// reference it later.
    pub fn add_texture(&mut self, texture: Texture) -> ManagedTextureId {
        self.textures.write().add(texture)
    }

    /// Remove a texture from the Paint DOM.
    pub fn remove_texture(&mut self, id: ManagedTextureId) {
        self.textures.write().remove(id)
    }

    /// Mark a texture as modified so that changes can be detected.
    pub fn mark_texture_modified(&mut self, id: ManagedTextureId) {
        self.textures.write().mark_modified(id);
    }

    /// Returns access to the PaintDom's texture storage.
    pub fn textures(&self) -> impl Deref<Target = Textures> + '_ {
        self.textures.read()
    }

    /// Returns mutable access to the PaintDom's texture storage.
    pub fn textures_mut(&self) -> impl DerefMut<Target = Textures> + '_ {
        self.textures.write()
    }

    /// Returns a list of layers that should be used to draw the UI.
    pub fn layers(&self) -> &PaintLayers {
        &self.layers
    }

    /// Add a mesh to be painted.
    pub fn add_mesh<V, I>(&mut self, mesh: PaintMesh<V, I>)
    where
        V: IntoIterator<Item = Vertex>,
        I: IntoIterator<Item = u16>,
    {
        profiling::scope!("PaintDom::add_mesh");

        let texture_id = mesh.texture.map(|(index, _rect)| index);

        let layer = self
            .layers
            .current_mut()
            .expect("an active layer is required to call add_mesh");

        let current_clip = self.clip_stack.last().copied();
        let call = match layer.calls.last_mut() {
            Some(call)
                if call.texture == texture_id
                    && call.pipeline == mesh.pipeline
                    && call.clip == current_clip =>
            {
                call
            }
            _ => {
                let mut call = PaintCall::new();
                call.texture = texture_id;
                call.pipeline = mesh.pipeline;
                call.clip = current_clip;

                layer.calls.push(call);
                layer.calls.last_mut().unwrap()
            }
        };

        let indices = mesh
            .indices
            .into_iter()
            .map(|index| index + call.vertices.len() as u16);
        call.indices.extend(indices);

        let vertices = mesh.vertices.into_iter().map(|mut vertex| {
            let mut pos = vertex.position * self.scale_factor;
            pos += self.unscaled_viewport.pos();

            // Currently, we only round the vertices of geometry fed to the text
            // pipeline because rounding all geometry causes hairline cracks in
            // some geometry, like rounded rectangles.
            //
            // See: https://github.com/SecondHalfGames/yakui/issues/153
            if mesh.pipeline == Pipeline::Text {
                pos = pos.round();
            }

            pos /= self.surface_size;

            vertex.position = pos;
            vertex
        });
        call.vertices.extend(vertices);
    }

    /// Use the given region as the clipping rect for all following paint calls.
    fn push_clip(&mut self, region: Rect) {
        let mut unscaled = Rect::from_pos_size(
            region.pos() * self.scale_factor,
            region.size() * self.scale_factor,
        );

        if let Some(previous) = self.clip_stack.last() {
            unscaled = unscaled.constrain(*previous);
        }

        self.clip_stack.push(unscaled);
    }

    /// Pop the most recent clip region, restoring the previous clipping rect.
    fn pop_clip(&mut self) {
        let top = self.clip_stack.pop();
        debug_assert!(
            top.is_some(),
            "cannot call pop_clip without a corresponding push_clip call"
        );
    }
}
