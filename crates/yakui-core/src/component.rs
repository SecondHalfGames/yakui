use std::any::{type_name, Any, TypeId};
use std::fmt;

use glam::Vec2;
use thunderdome::Index;

use crate::dom::{Dom, LayoutDom};
use crate::draw::Output;
use crate::geometry::Constraints;
use crate::input::MouseButton;

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

pub trait Component: Any + fmt::Debug {
    type Props: Props;
    type Response;

    fn new(index: Index, props: Self::Props) -> Self;
    fn update(&mut self, props: &Self::Props);
    fn size(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2;
    fn draw(&self, dom: &Dom, layout: &LayoutDom, output: &mut Output);
    fn respond(&mut self) -> Self::Response;

    fn event(&mut self, _event: &ComponentEvent) {}
}

pub trait ErasedComponent: Any {
    fn update(&mut self, props: &dyn ErasedProps);
    fn size(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2;
    fn draw(&self, dom: &Dom, layout: &LayoutDom, output: &mut Output);
    fn event(&mut self, event: &ComponentEvent);

    fn as_debug(&self) -> &dyn fmt::Debug;
}

impl<T> ErasedComponent for T
where
    T: Component,
{
    #[inline]
    fn update(&mut self, props: &dyn ErasedProps) {
        let props = props
            .downcast_ref::<T::Props>()
            .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T::Props>()));

        <T as Component>::update(self, props);
    }

    #[inline]
    fn size(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> Vec2 {
        <T as Component>::size(self, dom, layout, constraints)
    }

    #[inline]
    fn draw(&self, dom: &Dom, layout: &LayoutDom, output: &mut Output) {
        <T as Component>::draw(self, dom, layout, output)
    }

    #[inline]
    fn event(&mut self, event: &ComponentEvent) {
        <T as Component>::event(self, event)
    }

    #[inline]
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
}

mopmopafy!(ErasedComponent);

#[allow(clippy::enum_variant_names)]
pub enum ComponentEvent {
    MouseEnter,
    MouseLeave,
    MouseButtonChangedInside(MouseButton, bool),
    MouseButtonChangedOutside(MouseButton, bool),
}

// Placeholder component used internally.
#[derive(Debug)]
pub struct DummyComponent;

impl Component for DummyComponent {
    type Props = ();
    type Response = ();

    #[inline]
    fn new(_index: Index, _props: Self::Props) -> Self {
        Self
    }

    #[inline]
    fn update(&mut self, _props: &Self::Props) {}

    #[inline]
    fn event(&mut self, _event: &ComponentEvent) {}

    #[inline]
    fn size(&self, _dom: &Dom, _layout: &mut LayoutDom, _constraints: Constraints) -> Vec2 {
        Vec2::ZERO
    }

    #[inline]
    fn draw(&self, _dom: &Dom, _layout: &LayoutDom, _output: &mut crate::draw::Output) {}

    #[inline]
    fn respond(&mut self) {}
}
