use std::any::{Any, TypeId};
use std::fmt;

use glam::Vec2;
use thunderdome::Index;

use crate::dom::Dom;
use crate::layout::Constraints;

pub trait Props: Any + fmt::Debug {}

impl<T> Props for T where T: Any + fmt::Debug {}

pub trait Component: Any + fmt::Debug {
    type Props: Props;

    fn new(index: Index, props: &Self::Props) -> Self;
    fn update(&mut self, props: &Self::Props);
    fn size(&self, dom: &Dom, constraints: Constraints) -> Vec2;
}

#[derive(Clone, Copy)]
pub struct ComponentImpl {
    pub new: fn(index: Index, &dyn Any) -> Box<dyn ErasedComponent>,
    pub update: fn(&mut dyn ErasedComponent, &dyn Any),
    pub size: fn(&dyn ErasedComponent, &Dom, Constraints) -> Vec2,

    pub debug: fn(&dyn ErasedComponent) -> &dyn fmt::Debug,
    pub debug_props: fn(&dyn Any) -> &dyn fmt::Debug,
}

impl ComponentImpl {
    pub fn new<T: Component>() -> ComponentImpl {
        todo!()
    }
}

pub trait ErasedComponent: Any {}
impl<T> ErasedComponent for T where T: Component {}

impl dyn ErasedComponent {
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        let t = TypeId::of::<T>();
        let concrete = self.type_id();
        t == concrete
    }

    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        debug_assert!(self.is::<T>());
        &*(self as *const dyn ErasedComponent as *const T)
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
        debug_assert!(self.is::<T>());
        &mut *(self as *mut dyn ErasedComponent as *mut T)
    }
}

// Placeholder component used internally.
#[derive(Debug)]
pub struct DummyComponent;

impl Component for DummyComponent {
    type Props = ();

    fn new(_index: Index, _props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) {}

    fn size(&self, _dom: &Dom, _constraints: Constraints) -> Vec2 {
        Vec2::ZERO
    }
}
