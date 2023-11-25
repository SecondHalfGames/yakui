use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use glam::Vec2;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::Constraints;
use yakui_core::widget::{EventContext, LayoutContext, Widget};
use yakui_core::Yakui;

#[derive(Debug)]
struct TestWidget;

impl Widget for TestWidget {
    type Props<'a> = ();
    type Response = ();

    fn new() -> Self {
        Self
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}
}

/// https://github.com/LPGhatguy/yakui/issues/69
#[test]
fn layout_nodes_leak() {
    let mut yak = Yakui::new();

    yak.start();
    yak.dom().do_widget::<TestWidget>(());
    yak.dom().do_widget::<TestWidget>(());
    yak.finish();

    let dom = yak.dom();
    let layout = yak.layout_dom();
    assert_eq!(dom.len(), 3); // Two of our nodes, plus the root node.
    assert_eq!(layout.len(), 3);

    yak.start();
    yak.finish();

    let dom = yak.dom();
    let layout = yak.layout_dom();
    assert_eq!(dom.len(), 1); // Just the root node now
    assert_eq!(layout.len(), 1);
}

#[derive(Debug)]
struct KeyboardWidget {
    count: Rc<AtomicUsize>,
}

impl Widget for KeyboardWidget {
    type Props<'a> = ();
    type Response = Rc<AtomicUsize>;

    fn new() -> Self {
        Self {
            count: Rc::new(AtomicUsize::new(0)),
        }
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {
        self.count.clone()
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::FOCUSED_KEYBOARD
    }

    fn layout(&self, ctx: LayoutContext<'_>, _constraints: Constraints) -> Vec2 {
        ctx.input.set_selection(Some(ctx.dom.current()));
        Vec2::ZERO
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::TextInput(_) = event {
            self.count.fetch_add(1, Ordering::SeqCst);
        }

        EventResponse::Bubble
    }
}

/// https://github.com/LPGhatguy/yakui/issues/38
#[test]
fn input_events_to_removed_widgets() {
    let mut yak = Yakui::new();

    yak.start();
    let count = yak.dom().do_widget::<KeyboardWidget>(());
    yak.finish();

    assert_eq!(count.load(Ordering::SeqCst), 0);
    yak.handle_event(yakui_core::event::Event::TextInput('h'));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    yak.handle_event(yakui_core::event::Event::TextInput('e'));
    assert_eq!(count.load(Ordering::SeqCst), 2);

    yak.start();
    yak.finish();

    yak.handle_event(yakui_core::event::Event::TextInput('l'));
    assert_eq!(count.load(Ordering::SeqCst), 2);
}
