//! Defines yakui's layout protocol and Layout DOM.

mod clipping;

pub use self::clipping::*;

use std::collections::VecDeque;

use glam::Vec2;
use thunderdome::Arena;

use crate::dom::Dom;
use crate::event::EventInterest;
use crate::geometry::{Constraints, Rect};
use crate::id::WidgetId;
use crate::input::{InputState, MouseInterest};
use crate::paint::PaintDom;
use crate::widget::LayoutContext;

/// Contains information on how each widget in the DOM is laid out and what
/// events they're interested in.
#[derive(Debug)]
pub struct LayoutDom {
    nodes: Arena<LayoutDomNode>,

    unscaled_viewport: Rect,
    scale_factor: f32,

    pub(crate) interest_mouse: MouseInterest,

    clip_logic_overrides: Arena<ClipLogic>,
}

/// A node in a [`LayoutDom`].
#[derive(Debug)]
pub struct LayoutDomNode {
    /// The bounding rectangle of the node in logical pixels.
    pub rect: Rect,

    /// The clipping rectangle of the node in logical pixels.
    pub clip: Rect,

    /// This node is the beginning of a new layer, and all of its descendants
    /// should be hit tested and painted with higher priority.
    pub new_layer: bool,

    /// What events the widget reported interest in.
    pub event_interest: EventInterest,
}

impl LayoutDom {
    /// Create an empty `LayoutDom`.
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),

            unscaled_viewport: Rect::ONE,
            scale_factor: 1.0,

            interest_mouse: MouseInterest::new(),
            clip_logic_overrides: Arena::new(),
        }
    }

    pub(crate) fn sync_removals(&mut self, removals: &[WidgetId]) {
        for id in removals {
            self.nodes.remove(id.index());
        }
    }

    /// Get a widget's layout information.
    pub fn get(&self, id: WidgetId) -> Option<&LayoutDomNode> {
        self.nodes.get(id.index())
    }

    /// Get a mutable reference to a widget's layout information.
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut LayoutDomNode> {
        self.nodes.get_mut(id.index())
    }

    /// Set the viewport of the DOM in unscaled units.
    pub fn set_unscaled_viewport(&mut self, view: Rect) {
        self.unscaled_viewport = view;
    }

    /// Set the scale factor to use for layout.
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;
    }

    /// Get the currently active scale factor.
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Get the viewport in scaled units.
    pub fn viewport(&self) -> Rect {
        Rect::from_pos_size(
            self.unscaled_viewport.pos() / self.scale_factor,
            self.unscaled_viewport.size() / self.scale_factor,
        )
    }

    /// Get the viewport in unscaled units.
    pub fn unscaled_viewport(&self) -> Rect {
        self.unscaled_viewport
    }

    /// Tells how many nodes are currently in the `LayoutDom`.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Tells whether the `LayoutDom` is currently empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Calculate the layout of all elements in the given DOM.
    pub fn calculate_all(&mut self, dom: &Dom, input: &InputState, paint: &PaintDom) {
        profiling::scope!("LayoutDom::calculate_all");
        log::debug!("LayoutDom::calculate_all()");

        self.interest_mouse.clear();
        self.clip_logic_overrides.clear();

        let constraints = Constraints::tight(self.viewport().size());

        self.calculate(dom, input, paint, dom.root(), constraints);
        self.resolve_positions(dom);
        self.resolve_clipping(dom);
    }

    /// Calculate the layout of a specific widget.
    ///
    /// This function must only be called from
    /// [`Widget::layout`][crate::widget::Widget::layout] and should only be
    /// called once per widget per layout pass.
    pub fn calculate(
        &mut self,
        dom: &Dom,
        input: &InputState,
        paint: &PaintDom,
        id: WidgetId,
        constraints: Constraints,
    ) -> Vec2 {
        dom.enter(id);
        let dom_node = dom.get(id).unwrap();

        let context = LayoutContext {
            dom,
            input,
            layout: self,
            paint,
        };

        let size = dom_node.widget.layout(context, constraints);

        // If the widget called new_layer() during layout, it will be on top of
        // the mouse interest layer stack.
        let new_layer = self.interest_mouse.current_layer_root() == Some(id);

        // Mouse interest will be registered into the layout created by the
        // widget if there is one.
        let event_interest = dom_node.widget.event_interest();
        if event_interest.intersects(EventInterest::MOUSE_ALL) {
            self.interest_mouse.insert(id, event_interest);
        }

        // If the widget created a new layer, we're done with it now, so it's
        // time to clean it up.
        if new_layer {
            self.interest_mouse.pop_layer();
        }

        self.nodes.insert_at(
            id.index(),
            LayoutDomNode {
                rect: Rect::from_pos_size(Vec2::ZERO, size),
                clip: Rect::ZERO,
                new_layer,
                event_interest,
            },
        );

        dom.exit(id);
        size
    }

    /// Sets the clipping logic for the currently active widget.
    pub fn set_clip_logic(&mut self, dom: &Dom, logic: ClipLogic) {
        self.clip_logic_overrides
            .insert_at(dom.current().index(), logic);
    }

    /// Enables clipping for the currently active widget.
    pub fn enable_clipping(&mut self, dom: &Dom) {
        self.set_clip_logic(
            dom,
            ClipLogic::Constrain(AbstractClipRect::ParentClip, AbstractClipRect::LayoutRect),
        );
    }

    /// Escapes clipping from the current clipping rect for the currently active widget.
    pub fn escape_clipping(&mut self, dom: &Dom) {
        self.set_clip_logic(dom, ClipLogic::Override(AbstractClipRect::Viewport));
    }

    /// Put this widget and its children into a new layer.
    pub fn new_layer(&mut self, dom: &Dom) {
        self.interest_mouse.push_layer(dom.current());
    }

    /// Set the position of a widget.
    pub fn set_pos(&mut self, id: WidgetId, pos: Vec2) {
        if let Some(node) = self.nodes.get_mut(id.index()) {
            node.rect.set_pos(pos);
        }
    }

    fn resolve_positions(&mut self, dom: &Dom) {
        let mut queue = VecDeque::new();

        queue.push_back((dom.root(), self.viewport().pos()));

        while let Some((id, parent_pos)) = queue.pop_front() {
            if let Some(layout_node) = self.nodes.get_mut(id.index()) {
                let node = dom.get(id).unwrap();
                layout_node
                    .rect
                    .set_pos(layout_node.rect.pos() + parent_pos);

                queue.extend(node.children.iter().map(|&id| (id, layout_node.rect.pos())));
            }
        }
    }

    fn resolve_clipping(&mut self, dom: &Dom) {
        let viewport = self.viewport();

        let mut queue = VecDeque::new();

        queue.push_back((dom.root(), viewport, viewport, Vec2::ZERO));

        while let Some((id, parent_clip, parent_rect, mut offset)) = queue.pop_front() {
            let Some(layout_node) = self.nodes.get_mut(id.index()) else {
                continue;
            };
            let node = dom.get(id).unwrap();

            let logic = self
                .clip_logic_overrides
                .get(id.index())
                .copied()
                .unwrap_or(ClipLogic::Pass);

            let layout_rect = layout_node.rect;
            let mut new_layout_rect =
                Rect::from_pos_size(layout_rect.pos() + offset, layout_rect.size());

            // We need to do this before the clipping rect resolution.
            // We might change the layout rect here, and that might change what the clipping should be.
            if let Some(rect) = logic.resolve_layout(ClipResolutionArgs {
                parent_clip,
                parent_rect,
                layout_rect,
                viewport,
                offset,
            }) {
                offset += rect.pos() - layout_rect.pos();
                new_layout_rect = rect;
            }

            let clip_rect = logic.resolve_clip(ClipResolutionArgs {
                parent_clip,
                parent_rect,
                layout_rect,
                viewport,
                offset,
            });

            layout_node.clip = clip_rect;
            layout_node.rect = new_layout_rect;

            queue.extend(
                node.children
                    .iter()
                    .map(|&id| (id, layout_node.clip, layout_node.rect, offset)),
            );
        }
    }
}
