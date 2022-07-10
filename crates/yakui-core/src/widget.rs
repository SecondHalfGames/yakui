use std::any::{Any, TypeId};
use std::fmt;

use glam::Vec2;
use thunderdome::Index;

use crate::dom::Dom;
use crate::geometry::Constraints;
use crate::input::MouseButton;
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

    fn new(index: Index, props: Self::Props) -> Self;
    fn update(&mut self, props: Self::Props);
    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2;
    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom);
    fn respond(&mut self) -> Self::Response;

    fn event(&mut self, _event: &WidgetEvent) {}
}

pub trait ErasedWidget: Any {
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

#[allow(clippy::enum_variant_names)]
#[non_exhaustive]
pub enum WidgetEvent {
    MouseEnter,
    MouseLeave,
    MouseButtonChangedInside(MouseButton, bool),
    MouseButtonChangedOutside(MouseButton, bool),
}

// Placeholder widget used internally.
#[derive(Debug)]
pub struct DummyWidget;

impl Widget for DummyWidget {
    type Props = ();
    type Response = ();

    #[inline]
    fn new(_index: Index, _props: Self::Props) -> Self {
        Self
    }

    #[inline]
    fn update(&mut self, _props: Self::Props) {}

    #[inline]
    fn event(&mut self, _event: &WidgetEvent) {}

    #[inline]
    fn layout(&self, _dom: &Dom, _layout: &mut LayoutDom, _constraints: Constraints) -> Vec2 {
        Vec2::ZERO
    }

    #[inline]
    fn paint(&self, _dom: &Dom, _layout: &LayoutDom, _paint: &mut PaintDom) {}

    #[inline]
    fn respond(&mut self) {}
}
