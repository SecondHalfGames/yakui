//! Defines traits for building widgets.

use std::any::{type_name, Any, TypeId};
use std::fmt;

use glam::Vec2;

use crate::dom::Dom;
use crate::event::EventResponse;
use crate::event::{EventInterest, WidgetEvent};
use crate::geometry::{Constraints, FlexFit};
use crate::layout::LayoutDom;
use crate::paint::PaintDom;

/// Trait that's automatically implemented for all widget props.
///
/// This trait is used by yakui to enforce that props hold no non-`'static`
/// references and implement `Debug`.
pub trait Props: 'static + fmt::Debug {}
impl<T> Props for T where T: 'static + fmt::Debug {}

/// A yakui widget. Implement this trait to create a custom widget if composing
/// existing widgets does not solve your use case.
pub trait Widget: 'static + fmt::Debug {
    /// The props that this widget needs to be created or updated. Props define
    /// all of the values that a widget's user can specify every render.
    type Props: Props;

    /// The type that the widget will return to the user when it is created or
    /// updated. This type should contain information like whether the widget
    /// was clicked, had keyboard input, or other info that might be useful.
    type Response;

    /// Create the widget.
    fn new() -> Self;

    /// Update the widget with new props.
    fn update(&mut self, props: Self::Props) -> Self::Response;

    /// Returns whether this widget should grow to fill a flexible layout, and
    /// if so, what weight should be applied to it if other widgets also want to
    /// grow.
    ///
    /// A value of `0` indicates that this widget should not grow, while `1`
    /// means that it should stretch to take the available space.
    fn flex(&self) -> (u32, FlexFit) {
        (0, FlexFit::Loose)
    }

    /// Calculate this widget's layout with the given constraints and return its
    /// size. The returned size must fit within the given constraints, which can
    /// be done using `constraints.constrain(size)`.
    ///
    /// The default implementation will lay out all of this widget's children on
    /// top of each other, and fit the widget tightly around them.
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        let node = dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = layout.calculate(dom, child, constraints);
            size = size.max(child_size);
        }

        constraints.constrain_min(size)
    }

    /// Paint the widget based on its current state.
    ///
    /// The default implementation will paint all of the widget's children.
    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }

    /// Tells which events the widget is interested in receiving.
    ///
    /// The default implementation will register interest in no events.
    fn event_interest(&self) -> EventInterest {
        EventInterest::empty()
    }

    /// Handle the given event and update the widget's state.
    ///
    /// The default implementation will bubble all events.
    #[allow(unused)]
    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        EventResponse::Bubble
    }
}

/// A type-erased version of [`Widget`].
pub trait ErasedWidget: Any + fmt::Debug {
    /// See [`Widget::layout`].
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2;

    /// See [`Widget::flex`].
    fn flex(&self) -> (u32, FlexFit);

    /// See [`Widget::paint`].
    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom);

    /// See [`Widget::event_interest`].
    fn event_interest(&self) -> EventInterest;

    /// See [`Widget::event`].
    fn event(&mut self, event: &WidgetEvent) -> EventResponse;

    /// Returns the type name of the widget, usable only for debugging.
    fn type_name(&self) -> &'static str;
}

impl<T> ErasedWidget for T
where
    T: Widget,
{
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        <T as Widget>::layout(self, dom, layout, constraints)
    }

    fn flex(&self) -> (u32, FlexFit) {
        <T as Widget>::flex(self)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        <T as Widget>::paint(self, dom, layout, paint)
    }

    fn event_interest(&self) -> EventInterest {
        <T as Widget>::event_interest(self)
    }

    fn event(&mut self, event: &WidgetEvent) -> EventResponse {
        log::debug!("Event on {}: {event:?}", type_name::<T>());

        <T as Widget>::event(self, event)
    }

    fn type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

mopmopafy!(ErasedWidget);
