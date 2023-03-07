//! Defines yakui's layout protocol and Layout DOM.

use std::collections::VecDeque;

use glam::Vec2;
use thunderdome::Arena;

use crate::dom::Dom;
use crate::event::EventInterest;
use crate::geometry::{Constraints, Rect};
use crate::id::WidgetId;
use crate::input::InputState;
use crate::widget::LayoutContext;

/// Contains information on how each widget in the DOM is laid out and what
/// events they're interested in.
#[derive(Debug)]
pub struct LayoutDom {
    nodes: Arena<LayoutDomNode>,
    clip_stack: Vec<WidgetId>,

    unscaled_viewport: Rect,
    scale_factor: f32,

    pub(crate) interest_mouse: Vec<(WidgetId, EventInterest)>,
}

/// A node in a [`LayoutDom`].
#[derive(Debug)]
#[non_exhaustive]
pub struct LayoutDomNode {
    /// The bounding rectangle of the node.
    pub rect: Rect,

    /// This node will clip its descendants to its bounding rectangle.
    pub clipping_enabled: bool,

    /// This node is clipped to the region defined by the given node.
    pub clipped_by: Option<WidgetId>,

    /// What events the widget reported interest in.
    pub event_interest: EventInterest,
}

impl LayoutDom {
    /// Create an empty `LayoutDom`.
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),
            clip_stack: Vec::new(),

            unscaled_viewport: Rect::ONE,
            scale_factor: 1.0,

            interest_mouse: Vec::new(),
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
    pub fn calculate_all(&mut self, dom: &Dom, input: &InputState) {
        profiling::scope!("LayoutDom::calculate_all");
        log::debug!("LayoutDom::calculate_all()");

        self.clip_stack.clear();
        self.interest_mouse.clear();

        let constraints = Constraints::tight(self.viewport().size());

        self.calculate(dom, input, dom.root(), constraints);
        self.resolve_positions(dom);
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
        id: WidgetId,
        constraints: Constraints,
    ) -> Vec2 {
        dom.enter(id);
        let dom_node = dom.get(id).unwrap();

        let context = LayoutContext {
            dom,
            input,
            layout: self,
        };

        let size = dom_node.widget.layout(context, constraints);
        let event_interest = dom_node.widget.event_interest();
        if event_interest.intersects(EventInterest::MOUSE_ALL) {
            self.interest_mouse.push((id, event_interest));
        }

        // If the widget called enable_clipping() during its layout pass, it
        // should be on top of the clip stack at this point.
        let clipping_enabled = self.clip_stack.last() == Some(&id);

        // If this node enabled clipping, the next node under that is the node
        // that clips this one.
        let clipped_by = if clipping_enabled {
            self.clip_stack.iter().nth_back(2).copied()
        } else {
            self.clip_stack.last().copied()
        };

        self.nodes.insert_at(
            id.index(),
            LayoutDomNode {
                rect: Rect::from_pos_size(Vec2::ZERO, size),
                clipping_enabled,
                clipped_by,
                event_interest,
            },
        );

        if clipping_enabled {
            self.clip_stack.pop();
        }

        dom.exit(id);
        size
    }

    /// Enables clipping for the currently active widget.
    pub fn enable_clipping(&mut self, dom: &Dom) {
        self.clip_stack.push(dom.current());
    }

    /// Set the position of a widget.
    pub fn set_pos(&mut self, id: WidgetId, pos: Vec2) {
        if let Some(node) = self.nodes.get_mut(id.index()) {
            node.rect.set_pos(pos);
        }
    }

    fn resolve_positions(&mut self, dom: &Dom) {
        let mut queue = VecDeque::new();

        queue.push_back((dom.root(), Vec2::ZERO));

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
}
