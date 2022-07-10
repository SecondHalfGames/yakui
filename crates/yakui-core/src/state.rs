use std::collections::VecDeque;
use std::mem::take;

use glam::Vec2;
use thunderdome::Index;

use crate::context::Context;
use crate::dom::Dom;
use crate::input::InputState;
use crate::layout::LayoutDom;
use crate::paint::{PaintDom, Texture};
use crate::widget::WidgetEvent;
use crate::{ButtonState, Event};

#[derive(Debug)]
pub struct State {
    dom: Option<Dom>,
    layout: LayoutDom,
    paint: PaintDom,
    input: InputState,
    last_mouse_hit: Vec<Index>,
    mouse_hit: Vec<Index>,
}

impl State {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            dom: Some(Dom::new()),
            layout: LayoutDom::new(),
            paint: PaintDom::new(),
            input: InputState::new(),
            last_mouse_hit: Vec::new(),
            mouse_hit: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        assert!(
            self.dom.is_some(),
            "Cannot handle_event() while DOM is being built."
        );

        match event {
            Event::SetViewport(viewport) => {
                self.layout.viewport = viewport;
            }
            Event::MoveMouse(pos) => {
                self.input.mouse_position = Some(pos);
            }
            Event::MouseButtonChanged(button, down) => {
                self.input.mouse_button_changed(button, down);
            }
        }
    }

    pub fn create_texture(&mut self, texture: Texture) -> Index {
        self.paint.create_texture(texture)
    }

    pub fn start(&mut self) {
        let context = Context::current();

        if let Some(mut dom) = self.dom.take() {
            dom.start();
            context.borrow_mut().start(dom);
        } else {
            panic!("Cannot call start() when already started.");
        }
    }

    pub fn finish(&mut self) {
        let context = Context::current();
        let mut context = context.borrow_mut();

        let dom = if let Some(dom) = context.take_dom() {
            self.dom.insert(dom)
        } else {
            panic!("Cannot call finish() when not started.");
        };

        self.layout.calculate_all(dom);

        let mut mouse_hit = take(&mut self.last_mouse_hit);
        mouse_hit.clear();
        self.last_mouse_hit = take(&mut self.mouse_hit);

        if let Some(mouse_pos) = self.input.mouse_position {
            hit_test(dom, &self.layout, mouse_pos, &mut mouse_hit);
        }
        self.mouse_hit = mouse_hit;

        // oops, quadratic behavior
        for &hit in &self.mouse_hit {
            if let Some(node) = dom.get_mut(hit) {
                if !self.last_mouse_hit.contains(&hit) {
                    node.widget.event(&WidgetEvent::MouseEnter);
                }

                for (&button, state) in self.input.mouse_buttons.iter() {
                    match state {
                        ButtonState::JustDown => node
                            .widget
                            .event(&WidgetEvent::MouseButtonChangedInside(button, true)),
                        ButtonState::JustUp => node
                            .widget
                            .event(&WidgetEvent::MouseButtonChangedInside(button, false)),
                        _ => (),
                    }
                }
            }
        }

        for &hit in &self.last_mouse_hit {
            if !self.mouse_hit.contains(&hit) {
                if let Some(node) = dom.get_mut(hit) {
                    node.widget.event(&WidgetEvent::MouseLeave);
                }
            }
        }

        self.input.step();
    }

    pub fn paint(&mut self) -> &PaintDom {
        let dom = self.dom.as_ref().unwrap_or_else(|| {
            panic!("Cannot paint() while DOM is being built.");
        });

        self.paint.paint(dom, &self.layout);
        &self.paint
    }

    pub fn textures(&self) -> impl Iterator<Item = (Index, &Texture)> {
        self.paint.textures()
    }
}

fn hit_test(dom: &Dom, layout: &LayoutDom, coords: Vec2, output: &mut Vec<Index>) {
    let mut queue = VecDeque::new();

    queue.extend(dom.roots());

    while let Some(index) = queue.pop_front() {
        let node = dom.get(index).unwrap();
        let layout = layout.get(index).unwrap();

        if layout.rect.contains_point(coords) {
            output.push(index);
            queue.extend(&node.children);
        }
    }
}
