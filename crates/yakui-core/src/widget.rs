use std::any::{Any, TypeId};
use std::fmt;

use glam::Vec2;

use crate::dom::Dom;
use crate::event::WidgetEvent;
use crate::geometry::Constraints;
use crate::layout::LayoutDom;
use crate::paint::PaintDom;

pub trait Props: Any + fmt::Debug {}
impl<T> Props for T where T: Any + fmt::Debug {}

pub trait ErasedProps: Any {
    fn as_debug(&self) -> &dyn fmt::Debug;
}

impl<T> ErasedProps for T
where
    T: Props,
{
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
}

mopmopafy!(ErasedProps);

pub trait Widget: Any + fmt::Debug {
    type Props: Props;
    type Response;

    fn new(props: Self::Props) -> Self;
    fn update(&mut self, props: Self::Props);
    fn respond(&mut self) -> Self::Response;

    fn children(&self) {}

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        let node = dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = layout.calculate(dom, child, constraints);
            size = size.max(child_size);
        }

        constraints.constrain(size)
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get_current();
        for &child in &node.children {
            paint.paint(dom, layout, child);
        }
    }

    fn event(&mut self, _event: &WidgetEvent) {}
}

pub trait ErasedWidget: Any {
    fn children(&self) {}
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2;
    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom);
    fn event(&mut self, event: &WidgetEvent);

    fn as_debug(&self) -> &dyn fmt::Debug;
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

    #[inline]
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
}

mopmopafy!(ErasedWidget);
