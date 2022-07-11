use std::any::TypeId;
use std::fmt;

use glam::Vec2;

use crate::dom::Dom;
use crate::event::WidgetEvent;
use crate::geometry::Constraints;
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

    /// Create the widget with the given props.
    fn new(props: Self::Props) -> Self;

    /// Update the widget with new props.
    fn update(&mut self, props: Self::Props);

    /// Return a response, which lets users receive information from the widget
    /// like whether it was clicked.
    fn respond(&mut self) -> Self::Response;

    /// Construct the widget's children, helpful for reusing functionality
    /// between widgets.
    fn children(&self) {}

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

        constraints.constrain(size)
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

    /// Handle the given event and update the widget's state.
    fn event(&mut self, _event: &WidgetEvent) {}
}

/// A type-erased version of [`Widget`].
pub trait ErasedWidget: 'static + fmt::Debug {
    /// See [`Widget::children`].
    fn children(&self) {}

    /// See [`Widget::layout`].
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2;

    /// See [`Widget::paint`].
    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom);

    /// See [`Widget::event`].
    fn event(&mut self, event: &WidgetEvent);
}

impl<T> ErasedWidget for T
where
    T: Widget,
{
    #[inline]
    fn children(&self) {
        <T as Widget>::children(self)
    }

    #[inline]
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        <T as Widget>::layout(self, dom, layout, constraints)
    }

    #[inline]
    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        <T as Widget>::paint(self, dom, layout, paint)
    }

    #[inline]
    fn event(&mut self, event: &WidgetEvent) {
        <T as Widget>::event(self, event)
    }
}

mopmopafy!(ErasedWidget);
