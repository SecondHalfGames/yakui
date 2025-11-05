use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use glam::Vec2;
use smallvec::SmallVec;

use crate::dom::{Dom, DomNode};
use crate::event::{Event, EventInterest, EventResponse, WidgetEvent};
use crate::geometry::Rect;
use crate::id::WidgetId;
use crate::layout::LayoutDom;
use crate::navigation::{navigate, NavDirection};
use crate::widget::EventContext;

use super::mouse::MouseButton;
use super::{KeyCode, Modifiers};

/// Holds yakui's input state, like cursor position, hovered, and selected
/// widgets.
#[derive(Debug)]
pub struct InputState {
    /// State for the mouse, like buttons and position.
    mouse: RefCell<Mouse>,

    /// State of the keyboard modifier keys
    modifiers: Cell<Modifiers>,

    /// Details about widgets and their mouse intersections.
    intersections: RefCell<Intersections>,

    /// The widget that is currently selected.
    selection: Cell<Option<WidgetId>>,

    /// The widget that was selected last frame.
    last_selection: Cell<Option<WidgetId>>,

    /// If there's a pending navigation event, it's stored here!
    pending_navigation: Cell<Option<NavDirection>>,

    /// If set, text input should be active.
    text_input_enabled: Cell<bool>,

    /// If there's a text input active with a cursor, this will be set and forwarded to the window.
    text_cursor: Cell<Option<Rect>>,
}

#[derive(Debug)]
struct Mouse {
    /// The current mouse position, or `None` if it's outside the window.
    position: Option<Vec2>,

    /// The state of each mouse button. If missing from the map, the button is
    /// up and has not yet been pressed.
    buttons: HashMap<MouseButton, ButtonState>,
}

#[derive(Debug)]
struct Intersections {
    /// All of the widgets with mouse interest that the current mouse position
    /// intersects with.
    ///
    /// All lists like this are stored with the deepest widgets first.
    mouse_hit: Vec<WidgetId>,

    /// All of the widgets that have had a mouse enter event sent to them
    /// without a corresponding mouse leave event yet. This is different from
    /// mouse_hit because hover events can be sunk by event handlers.
    mouse_entered: Vec<WidgetId>,

    /// All of the widgets that had a mouse enter event sent to them and then
    /// sunk it that are still being hovered. This helps us ensure that a widget
    /// that sunk a hover event will continue to occupy that space even if we
    /// don't send it more events.
    mouse_entered_and_sunk: Vec<WidgetId>,

    /// All widgets that had the corresponding mouse button pressed while the
    /// mouse cursor was over them.
    #[allow(unused)]
    mouse_down_in: HashMap<MouseButton, Vec<WidgetId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonState {
    JustDown,
    Down,
    JustUp,
    Up,
}

impl ButtonState {
    pub fn is_down(&self) -> bool {
        matches!(self, Self::JustDown | Self::Down)
    }

    pub fn settle(&mut self) {
        match self {
            Self::JustDown => {
                *self = Self::Down;
            }
            Self::JustUp => {
                *self = Self::Up;
            }
            _ => (),
        }
    }
}

impl InputState {
    /// Create a new, empty `InputState`.
    pub fn new() -> Self {
        Self {
            mouse: RefCell::new(Mouse {
                position: None,
                buttons: HashMap::new(),
            }),
            modifiers: Cell::new(Modifiers::default()),
            intersections: RefCell::new(Intersections {
                mouse_hit: Vec::new(),
                mouse_entered: Vec::new(),
                mouse_entered_and_sunk: Vec::new(),
                mouse_down_in: HashMap::new(),
            }),
            selection: Cell::new(None),
            last_selection: Cell::new(None),
            pending_navigation: Cell::new(None),
            text_input_enabled: Cell::new(false),
            text_cursor: Cell::new(None),
        }
    }

    /// Begin a new frame for input handling.
    pub fn start(&self, dom: &Dom, layout: &LayoutDom) {
        self.text_input_enabled.set(false);
        self.text_cursor.set(None);
        self.notify_selection(dom, layout);
    }

    /// Finish applying input events for this frame.
    pub fn finish(&self, dom: &Dom, layout: &LayoutDom) {
        self.settle_buttons();
        self.handle_navigation(dom, layout);
    }

    fn handle_navigation(&self, dom: &Dom, layout: &LayoutDom) {
        if let Some(dir) = self.pending_navigation.take() {
            if let Some(new_focus) = navigate(dom, layout, self, dir) {
                dom.request_focus(new_focus);
            }
        }
    }

    /// Enables text input. Should be called every update from a widget that
    /// expects text events when it's focused.
    pub fn enable_text_input(&self) {
        self.text_input_enabled.set(true);
    }

    /// Tells whether a widget is currently looking for text input, like a
    /// focused textbox.
    pub fn text_input_enabled(&self) -> bool {
        self.text_input_enabled.get()
    }

    /// Sets the text cursor. Should be called every update from an active text input.
    ///
    /// Should be in physical pixels.
    pub fn set_text_cursor(&self, cursor: Rect) {
        self.text_cursor.set(Some(cursor));
    }

    /// Gets the text cursor, if any.
    ///
    /// Should be in physical pixels.
    pub fn get_text_cursor(&self) -> Option<Rect> {
        self.text_cursor.get()
    }

    /// Returns the mouse position, or [`None`] if it's outside the window.
    pub fn mouse_pos(&self, layout: &LayoutDom) -> Option<Vec2> {
        self.mouse
            .borrow()
            .position
            .map(|pos| pos / layout.scale_factor())
    }

    /// Return the currently selected widget, if there is one.
    pub fn selection(&self) -> Option<WidgetId> {
        self.selection.get()
    }

    /// Set the currently selected widget.
    pub fn set_selection(&self, id: Option<WidgetId>) {
        self.selection.set(id);
    }

    /// Attempt to navigate in a direction within the UI.
    pub fn navigate(&self, dir: NavDirection) {
        self.pending_navigation.set(Some(dir));
    }

    pub(crate) fn handle_event(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        event: Event,
    ) -> EventResponse {
        match event {
            Event::CursorMoved(pos) => {
                self.mouse_moved(dom, layout, pos);
                EventResponse::Bubble
            }
            Event::MouseButtonChanged { button, down } => {
                let response = self.mouse_button_changed(dom, layout, button, down);

                // If no widgets elected to handle mouse button one going down,
                // we can should clear our selection.
                //
                // FIXME: Currently, this gets sunk by widgets that sink events
                // but don't do anything to the selection state with them. We
                // should figure out how to detect that case, like clicking an
                // Opaque widget.
                if response == EventResponse::Bubble {
                    if button == MouseButton::One && down {
                        self.set_selection(None);
                        self.notify_selection(dom, layout);
                    }
                }

                response
            }
            Event::MouseScroll { delta } => self.send_mouse_scroll(dom, layout, delta),
            Event::KeyChanged {
                key,
                down,
                modifiers,
            } => self.keyboard_key_changed(dom, layout, key, down, modifiers),
            Event::ModifiersChanged(modifiers) => self.modifiers_changed(modifiers),
            Event::TextInput(c) => self.text_input(dom, layout, c),
            Event::TextPreedit(text, position) => self.text_preedit(dom, layout, text, position),
            _ => EventResponse::Bubble,
        }
    }

    fn notify_selection(&self, dom: &Dom, layout: &LayoutDom) {
        let mut current = self.selection.get();
        let last = self.last_selection.get();

        if current == last {
            return;
        }

        if let Some(entered) = current {
            if let Some(mut node) = dom.get_mut(entered) {
                self.fire_event(
                    dom,
                    layout,
                    entered,
                    &mut node,
                    &WidgetEvent::FocusChanged(true),
                );
            } else {
                self.selection.set(None);
                current = None;
            }
        }

        if let Some(left) = last {
            if let Some(mut node) = dom.get_mut(left) {
                self.fire_event(
                    dom,
                    layout,
                    left,
                    &mut node,
                    &WidgetEvent::FocusChanged(false),
                );
            }
        }

        self.last_selection.set(current);
    }

    /// Signal that the mouse has moved.
    fn mouse_moved(&self, dom: &Dom, layout: &LayoutDom, pos: Option<Vec2>) {
        let pos = pos.map(|pos| pos - layout.unscaled_viewport().pos());

        {
            let mut mouse = self.mouse.borrow_mut();
            mouse.position = pos;
        }

        self.send_mouse_move(dom, layout);
        self.mouse_hit_test(dom, layout);
        self.send_mouse_enter(dom, layout);
        self.send_mouse_leave(dom, layout);
    }

    /// Signal that a mouse button's state has changed.
    fn mouse_button_changed(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        down: bool,
    ) -> EventResponse {
        {
            let mut mouse = self.mouse.borrow_mut();
            let state = mouse.buttons.entry(button).or_insert(ButtonState::Up);

            match (state.is_down(), down) {
                // If the state didn't actually change, leave the current value
                // alone.
                (true, true) | (false, false) => (),

                (false, true) => {
                    *state = ButtonState::JustDown;
                }

                (true, false) => {
                    *state = ButtonState::JustUp;
                }
            }
        }

        self.send_button_change(dom, layout, button, down)
    }

    fn keyboard_key_changed(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        key: KeyCode,
        down: bool,
        modifiers: Option<Modifiers>,
    ) -> EventResponse {
        let selected = self.selection.get();
        if let Some(id) = selected {
            let Some(layout_node) = layout.get(id) else {
                return EventResponse::Bubble;
            };

            if layout_node
                .event_interest
                .contains(EventInterest::FOCUSED_KEYBOARD)
            {
                // Panic safety: if this node is in the layout DOM, it must be
                // in the DOM.
                let mut node = dom.get_mut(id).unwrap();
                let event = WidgetEvent::KeyChanged {
                    key,
                    down,
                    modifiers: modifiers.unwrap_or(self.modifiers.get()),
                };
                return self.fire_event(dom, layout, id, &mut node, &event);
            }
        }

        EventResponse::Bubble
    }

    fn modifiers_changed(&self, modifiers: Modifiers) -> EventResponse {
        self.modifiers.set(modifiers);
        EventResponse::Bubble
    }

    fn text_preedit(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        text: String,
        position: Option<(usize, usize)>,
    ) -> EventResponse {
        let selected = self.selection.get();
        if let Some(id) = selected {
            let Some(layout_node) = layout.get(id) else {
                return EventResponse::Bubble;
            };

            if layout_node
                .event_interest
                .contains(EventInterest::FOCUSED_KEYBOARD)
            {
                // Panic safety: if this node is in the layout DOM, it must be
                // in the DOM.
                let mut node = dom.get_mut(id).unwrap();
                let event = WidgetEvent::TextPreedit(text, position);
                return self.fire_event(dom, layout, id, &mut node, &event);
            }
        }

        EventResponse::Bubble
    }

    fn text_input(&self, dom: &Dom, layout: &LayoutDom, c: char) -> EventResponse {
        let selected = self.selection.get();
        if let Some(id) = selected {
            let Some(layout_node) = layout.get(id) else {
                return EventResponse::Bubble;
            };

            if layout_node
                .event_interest
                .contains(EventInterest::FOCUSED_KEYBOARD)
            {
                // Panic safety: if this node is in the layout DOM, it must be
                // in the DOM.
                let mut node = dom.get_mut(id).unwrap();
                let event = WidgetEvent::TextInput(c, self.modifiers.get());
                return self.fire_event(dom, layout, id, &mut node, &event);
            }
        }

        EventResponse::Bubble
    }

    fn send_button_change(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        button: MouseButton,
        down: bool,
    ) -> EventResponse {
        let mouse = self.mouse.borrow();
        let intersections = self.intersections.borrow();
        let mut overall_response = EventResponse::Bubble;

        for &id in &intersections.mouse_hit {
            if let Some(mut node) = dom.get_mut(id) {
                let event = WidgetEvent::MouseButtonChanged {
                    button,
                    down,
                    inside: true,
                    position: mouse.position.unwrap_or(Vec2::ZERO) / layout.scale_factor(),
                    modifiers: self.modifiers.get(),
                };
                let response = self.fire_event(dom, layout, id, &mut node, &event);

                if response == EventResponse::Sink {
                    overall_response = response;
                    break;
                }
            }
        }

        for (id, interest) in layout.interest_mouse.iter() {
            if interest.contains(EventInterest::MOUSE_OUTSIDE)
                && !intersections.mouse_hit.contains(&id)
            {
                if let Some(mut node) = dom.get_mut(id) {
                    let event = WidgetEvent::MouseButtonChanged {
                        button,
                        down,
                        inside: false,
                        position: mouse.position.unwrap_or(Vec2::ZERO) / layout.scale_factor(),
                        modifiers: self.modifiers.get(),
                    };
                    self.fire_event(dom, layout, id, &mut node, &event);
                }
            }
        }

        overall_response
    }

    fn send_mouse_scroll(&self, dom: &Dom, layout: &LayoutDom, delta: Vec2) -> EventResponse {
        let intersections = self.intersections.borrow();

        let mut overall_response = EventResponse::Bubble;

        for &id in &intersections.mouse_hit {
            if let Some(mut node) = dom.get_mut(id) {
                let event = WidgetEvent::MouseScroll {
                    delta,
                    modifiers: self.modifiers.get(),
                };
                let response = self.fire_event(dom, layout, id, &mut node, &event);

                if response == EventResponse::Sink {
                    overall_response = response;
                    break;
                }
            }
        }

        overall_response
    }

    fn send_mouse_move(&self, dom: &Dom, layout: &LayoutDom) {
        let mouse = self.mouse.borrow();
        let pos = mouse.position.map(|pos| pos / layout.scale_factor());
        let event = WidgetEvent::MouseMoved(pos);

        for (id, interest) in layout.interest_mouse.iter() {
            if interest.intersects(EventInterest::MOUSE_MOVE) {
                if let Some(mut node) = dom.get_mut(id) {
                    self.fire_event(dom, layout, id, &mut node, &event);
                }
            }
        }
    }

    fn send_mouse_enter(&self, dom: &Dom, layout: &LayoutDom) {
        let mut intersections = self.intersections.borrow_mut();
        let intersections = &mut *intersections;

        for &hit in &intersections.mouse_hit {
            if let Some(mut node) = dom.get_mut(hit) {
                if !intersections.mouse_entered.contains(&hit) {
                    intersections.mouse_entered.push(hit);

                    let response =
                        self.fire_event(dom, layout, hit, &mut node, &WidgetEvent::MouseEnter);

                    if response == EventResponse::Sink {
                        intersections.mouse_entered_and_sunk.push(hit);
                        break;
                    }
                } else if intersections.mouse_entered_and_sunk.contains(&hit) {
                    // This widget was hovered previously, is still hovered, and
                    // sunk the mouse enter event before. In order to prevent
                    // erroneously hovering other widgets, continue sinking this
                    // event.
                    break;
                }
            }
        }
    }

    fn send_mouse_leave(&self, dom: &Dom, layout: &LayoutDom) {
        let mut intersections = self.intersections.borrow_mut();

        let mut to_remove = SmallVec::<[WidgetId; 4]>::new();

        for &hit in &intersections.mouse_entered {
            if !intersections.mouse_hit.contains(&hit) {
                if let Some(mut node) = dom.get_mut(hit) {
                    self.fire_event(dom, layout, hit, &mut node, &WidgetEvent::MouseLeave);
                }

                to_remove.push(hit);
            }
        }

        for remove in to_remove {
            intersections.mouse_entered.retain(|&id| id != remove);
            intersections
                .mouse_entered_and_sunk
                .retain(|&id| id != remove);
        }
    }

    fn mouse_hit_test(&self, dom: &Dom, layout: &LayoutDom) {
        let mut intersections = self.intersections.borrow_mut();
        let mouse = self.mouse.borrow();

        intersections.mouse_hit.clear();

        if let Some(mut mouse_pos) = mouse.position {
            mouse_pos /= layout.scale_factor();
            hit_test(dom, layout, mouse_pos, &mut intersections.mouse_hit);
        }
    }

    fn settle_buttons(&self) {
        let mut mouse = self.mouse.borrow_mut();

        for state in mouse.buttons.values_mut() {
            state.settle();
        }
    }

    /// Notify the widget of an event, pushing it onto the stack first to ensure
    /// that the DOM will have the correct widget at the top of the stack if
    /// queried.
    fn fire_event(
        &self,
        dom: &Dom,
        layout: &LayoutDom,
        id: WidgetId,
        node: &mut DomNode,
        event: &WidgetEvent,
    ) -> EventResponse {
        let context = EventContext {
            dom,
            layout,
            input: self,
        };

        dom.enter(id);
        let response = node.widget.event(context, event);
        dom.exit(id);

        response
    }
}

/// Calculate the set of widgets that are under the given point, sorted by
/// relative height, highest to lowest.
#[profiling::function]
fn hit_test(_dom: &Dom, layout: &LayoutDom, coords: Vec2, output: &mut Vec<WidgetId>) {
    for (id, _interest) in layout.interest_mouse.iter() {
        let Some(layout_node) = layout.get(id) else {
            continue;
        };

        let mut rect = layout_node.rect;
        let mut node = layout_node;
        while let Some(parent) = node.clipped_by {
            node = layout.get(parent).unwrap();
            rect = rect.constrain(node.rect);
        }

        if rect.contains_point(coords) {
            output.push(id);
        }
    }
}
