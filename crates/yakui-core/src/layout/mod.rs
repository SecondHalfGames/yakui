use std::collections::VecDeque;

use glam::Vec2;
use thunderdome::Arena;

use crate::dom::Dom;
use crate::event::EventInterest;
use crate::geometry::{Constraints, Rect};
use crate::id::WidgetId;

#[derive(Debug)]
pub struct LayoutDom {
    nodes: Arena<LayoutDomNode>,

    viewport: Rect,
    scaled_viewport: Rect,
    scale_factor: f32,

    pub interest_mouse: Vec<(WidgetId, EventInterest)>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LayoutDomNode {
    pub rect: Rect,
    pub event_interest: EventInterest,
}

impl LayoutDom {
    pub fn new() -> Self {
        Self {
            nodes: Arena::new(),

            viewport: Rect::ONE,
            scaled_viewport: Rect::ONE,
            scale_factor: 1.0,

            interest_mouse: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn get(&self, id: WidgetId) -> Option<&LayoutDomNode> {
        self.nodes.get(id.index())
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut LayoutDomNode> {
        self.nodes.get_mut(id.index())
    }

    pub fn set_unscaled_viewport(&mut self, view: Rect) {
        self.viewport = view;
        self.scaled_viewport = Rect::from_pos_size(view.pos(), view.size() / self.scale_factor);
    }

    pub fn set_scale_factor(&mut self, scale: f32) {
        self.scale_factor = scale;

        let view = self.viewport;
        self.scaled_viewport = Rect::from_pos_size(view.pos(), view.size() / self.scale_factor);
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn viewport(&self) -> Rect {
        self.scaled_viewport
    }

    pub fn calculate_all(&mut self, dom: &Dom) {
        profiling::scope!("LayoutDom::calculate_all");
        log::debug!("LayoutDom::calculate_all()");

        self.interest_mouse.clear();

        let constraints = Constraints {
            min: Vec2::ZERO,
            max: self.viewport.size() / self.scale_factor,
        };

        self.calculate(dom, dom.root(), constraints);
        self.resolve_positions(dom);
    }

    pub fn calculate(&mut self, dom: &Dom, id: WidgetId, constraints: Constraints) -> Vec2 {
        dom.enter(id);
        let dom_node = dom.get(id).unwrap();
        let size = dom_node.widget.layout(dom, self, constraints);
        let event_interest = dom_node.widget.event_interest();

        if event_interest.intersects(EventInterest::MOUSE) {
            self.interest_mouse.push((id, event_interest));
        }

        self.nodes.insert_at(
            id.index(),
            LayoutDomNode {
                rect: Rect::from_pos_size(Vec2::ZERO, size),
                event_interest,
            },
        );
        dom.exit(id);
        size
    }

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
