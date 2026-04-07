use std::collections::HashMap;

use glam::Vec2;
use thunderdome::Arena;

use crate::dom::Dom;
use crate::geometry::Rect;
use crate::id::{ManagedTextureId, WidgetId};
use crate::layout::LayoutDom;
use crate::paint::{PaintCall, Pipeline, YakuiPaintCall};
use crate::widget::PaintContext;
use crate::Globals;

use super::layers::PaintLayers;
use super::primitives::{PaintMesh, Vertex};
use super::texture::{Texture, TextureChange};
use super::UserPaintCallId;

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

#[derive(Debug, Clone, Copy)]
/// Contains misc info about the surface to be painted onto.
pub struct PaintInfo {
    surface_size: Vec2,
    unscaled_viewport: Rect,
    scale_factor: f32,
}

impl PaintInfo {
    /// Transforms the vertex position from unit in logical pixels to a normalized position for rendering.
    pub fn transform_vertex(&self, pos: Vec2, round_to_physical_pixel: bool) -> Vec2 {
        transform_vertex(
            pos,
            self.scale_factor,
            self.surface_size,
            round_to_physical_pixel,
        )
    }
}

impl Default for PaintInfo {
    fn default() -> Self {
        Self {
            surface_size: Vec2::ONE,
            unscaled_viewport: Rect::ONE,
            scale_factor: 1.0,
        }
    }
}

/// Contains all information about how to paint the current set of widgets.
#[derive(Debug)]
pub struct PaintDom {
    textures: Arena<Texture>,
    texture_edits: HashMap<ManagedTextureId, TextureChange>,

    limits: Option<PaintLimits>,

    /// Contains misc info about the surface to be painted onto.
    pub info: PaintInfo,

    /// Stores the list of layers that should be used to draw the UI.
    pub layers: PaintLayers,

    /// Stores paint-persistent states.
    /// For things like custom renderers, for example.
    pub globals: Globals,

    current_clip: Rect,

    #[cfg(debug_assertions)]
    painted_already: bool,
}

impl PaintDom {
    /// Create a new, empty Paint DOM.
    pub fn new() -> Self {
        Self {
            textures: Arena::new(),
            texture_edits: HashMap::new(),

            limits: None,

            info: PaintInfo::default(),
            layers: PaintLayers::new(),
            globals: Globals::new(),

            current_clip: Rect::ZERO,

            #[cfg(debug_assertions)]
            painted_already: false,
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
        #[cfg(debug_assertions)]
        {
            self.painted_already = false;
        }

        self.texture_edits.clear();
    }

    /// Get the size of the surface that is being painted onto.
    pub fn surface_size(&self) -> Vec2 {
        self.info.surface_size
    }

    /// Set the size of the surface that is being painted onto.
    pub(crate) fn set_surface_size(&mut self, size: Vec2) {
        self.info.surface_size = size;
    }

    /// Get the viewport in unscaled units.
    pub fn unscaled_viewport(&self) -> Rect {
        self.info.unscaled_viewport
    }

    /// Set the viewport in unscaled units.
    pub(crate) fn set_unscaled_viewport(&mut self, viewport: Rect) {
        self.info.unscaled_viewport = viewport;
    }

    /// Get the currently active scale factor.
    pub fn scale_factor(&self) -> f32 {
        self.info.scale_factor
    }

    /// Set the currently active scale factor.
    pub(crate) fn set_scale_factor(&mut self, scale_factor: f32) {
        self.info.scale_factor = scale_factor;
    }

    /// Paint a specific widget. This function is usually called as part of an
    /// implementation of [`Widget::paint`][crate::widget::Widget::paint].
    ///
    /// Must only be called once per widget per paint pass.
    pub fn paint(&mut self, dom: &Dom, layout: &LayoutDom, id: WidgetId) {
        profiling::scope!("PaintDom::paint");

        let layout_node = layout.get(id).unwrap();
        if !layout_node.clip.intersects(&layout_node.rect) {
            return;
        }

        self.current_clip = Rect::from_pos_size(
            (layout_node.clip.pos() * self.scale_factor()).round(),
            (layout_node.clip.size() * self.scale_factor()).round(),
        )
        .constrain(layout.unscaled_viewport());

        if layout_node.new_layer {
            self.layers.push();
        }

        dom.enter(id);

        let context = PaintContext {
            dom,
            layout,
            clip: self.current_clip,
            paint: self,
        };
        let node = dom.get(id).unwrap();
        node.widget.paint(context);

        dom.exit(id);

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

        #[cfg(debug_assertions)]
        {
            debug_assert!(
                !self.painted_already,
                "PaintDom::paint_all() should only be called once per frame"
            );

            self.painted_already = true;
        }
    }

    /// Add a texture to the Paint DOM, returning an ID that can be used to
    /// reference it later.
    pub fn add_texture(&mut self, texture: Texture) -> ManagedTextureId {
        let id = ManagedTextureId::new(self.textures.insert(texture));
        self.texture_edits.insert(id, TextureChange::Added);
        id
    }

    /// Remove a texture from the Paint DOM.
    pub fn remove_texture(&mut self, id: ManagedTextureId) {
        self.textures.remove(id.index());
        self.texture_edits.insert(id, TextureChange::Removed);
    }

    /// Retrieve a texture by its ID, if it exists.
    pub fn texture(&self, id: ManagedTextureId) -> Option<&Texture> {
        self.textures.get(id.index())
    }

    /// Retrieves a mutable reference to a texture by its ID.
    pub fn texture_mut(&mut self, id: ManagedTextureId) -> Option<&mut Texture> {
        self.textures.get_mut(id.index())
    }

    /// Mark a texture as modified so that changes can be detected.
    pub fn mark_texture_modified(&mut self, id: ManagedTextureId) {
        self.texture_edits.insert(id, TextureChange::Modified);
    }

    /// Returns an iterator over all textures known to the Paint DOM.
    pub fn textures(&self) -> impl Iterator<Item = (ManagedTextureId, &Texture)> {
        self.textures
            .iter()
            .map(|(index, texture)| (ManagedTextureId::new(index), texture))
    }

    /// Iterates over the list of changes that happened to yakui-managed
    /// textures this frame.
    ///
    /// This is useful for renderers that need to upload or remove GPU resources
    /// related to textures.
    pub fn texture_edits(&self) -> impl Iterator<Item = (ManagedTextureId, TextureChange)> + '_ {
        self.texture_edits.iter().map(|(&id, &edit)| (id, edit))
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

        let call = match layer.calls.last_mut() {
            Some((clip, PaintCall::Internal(call)))
                if call.texture == texture_id
                    && call.pipeline == mesh.pipeline
                    && *clip == self.current_clip =>
            {
                call
            }
            _ => {
                let mut call = YakuiPaintCall::new();
                call.texture = texture_id;
                call.pipeline = mesh.pipeline;

                layer
                    .calls
                    .push((self.current_clip, PaintCall::Internal(call)));

                let Some((_, PaintCall::Internal(inserted))) = layer.calls.last_mut() else {
                    panic!()
                };

                inserted
            }
        };

        let indices = mesh
            .indices
            .into_iter()
            .map(|index| index + call.vertices.len() as u16);
        call.indices.extend(indices);

        let vertices = mesh.vertices.into_iter().map(|mut vertex| {
            // Currently, we only round the vertices of geometry fed to the text
            // pipeline because rounding all geometry causes hairline cracks in
            // some geometry, like rounded rectangles.
            //
            // See: https://github.com/SecondHalfGames/yakui/issues/153
            let round = mesh.pipeline == Pipeline::Text;

            vertex.position = self.info.transform_vertex(vertex.position, round);

            vertex
        });
        call.vertices.extend(vertices);
    }

    /// Adds a user-managed paint call to be painted.
    /// This expects the user to handle the paint call on their own, yakui just records the ID given to be checked against by the user later.
    pub fn add_user_call(&mut self, call_id: UserPaintCallId) {
        profiling::scope!("PaintDom::add_user_call");

        let layer = self
            .layers
            .current_mut()
            .expect("an active layer is required to call add_user_call");

        layer
            .calls
            .push((self.current_clip, PaintCall::User(call_id)));
    }
}

fn transform_vertex(
    mut pos: Vec2,
    scale_factor: f32,
    surface_size: Vec2,
    round_to_physical_pixel: bool,
) -> Vec2 {
    pos *= scale_factor;

    if round_to_physical_pixel {
        pos = pos.round();
    }

    pos /= surface_size;

    pos
}
