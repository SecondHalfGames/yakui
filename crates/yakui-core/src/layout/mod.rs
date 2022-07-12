use std::collections::VecDeque;

use glam::Vec2;
use thunderdome::{Arena, Index};

use crate::dom::Dom;
use crate::geometry::{Constraints, Rect};
use crate::EventInterest;

#[derive(Debug)]
pub struct LayoutDom {
    nodes: Arena<LayoutDomNode>,

    viewport: Rect,
    scaled_viewport: Rect,
    scale_factor: f32,

    pub interest_mouse: Vec<Index>,
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

    pub fn get(&self, index: Index) -> Option<&LayoutDomNode> {
        self.nodes.get(index)
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut LayoutDomNode> {
        self.nodes.get_mut(index)
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

    pub fn calculate(&mut self, dom: &Dom, index: Index, constraints: Constraints) -> Vec2 {
        dom.enter(index);
        let dom_node = dom.get(index).unwrap();
        let size = dom_node.widget.layout(dom, self, constraints);
        let event_interest = dom_node.widget.event_interest();

        if event_interest.intersects(EventInterest::MOUSE) {
            self.interest_mouse.push(index);
        }

        self.nodes.insert_at(
            index,
            LayoutDomNode {
                rect: Rect::from_pos_size(Vec2::ZERO, size),
                event_interest,
            },
        );
        dom.exit(index);
        size
    }

    pub fn set_pos(&mut self, index: Index, pos: Vec2) {
        if let Some(node) = self.nodes.get_mut(index) {
            node.rect.set_pos(pos);
        }
    }

    fn resolve_positions(&mut self, dom: &Dom) {
        let mut queue = VecDeque::new();

        queue.push_back((dom.root(), Vec2::ZERO));

        while let Some((index, parent_pos)) = queue.pop_front() {
            if let Some(layout_node) = self.nodes.get_mut(index) {
                let node = dom.get(index).unwrap();
                layout_node
                    .rect
                    .set_pos(layout_node.rect.pos() + parent_pos);

                queue.extend(
                    node.children
                        .iter()
                        .map(|&index| (index, layout_node.rect.pos())),
                );
            }
        }
    }
}
