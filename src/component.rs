use std::any::{type_name, Any, TypeId};
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
    pub fn new<T: Component>() -> Self {
        Self {
            new: new::<T>,
            update: update::<T>,
            size: size::<T>,
            debug: debug::<T>,
            debug_props: debug_props::<T::Props>,
        }
    }
}

fn new<T>(index: Index, props: &dyn Any) -> Box<dyn ErasedComponent>
where
    T: Component,
{
    let props = props.downcast_ref::<T::Props>().unwrap_or_else(|| {
        panic!(
            "Component {} expects props of type {} (ID {:?}), got ID {:?}",
            type_name::<T>(),
            type_name::<T::Props>(),
            TypeId::of::<T::Props>(),
            props.type_id(),
        )
    });

    let value: T = T::new(index, props);
    let boxed: Box<dyn ErasedComponent> = Box::new(value);
    boxed
}

fn update<T>(target: &mut dyn ErasedComponent, props: &dyn Any)
where
    T: Component,
{
    let target = target
        .downcast_mut::<T>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T>()));

    let props = props.downcast_ref::<T::Props>().unwrap_or_else(|| {
        panic!(
            "Component {} expects props of type {}",
            type_name::<T>(),
            type_name::<T::Props>()
        )
    });

    T::update(target, props);
}

fn size<T>(target: &dyn ErasedComponent, dom: &Dom, constraints: Constraints) -> Vec2
where
    T: Component,
{
    let target = target
        .downcast_ref::<T>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T>()));

    target.size(dom, constraints)
}

fn debug<T>(target: &dyn ErasedComponent) -> &dyn fmt::Debug
where
    T: Component,
{
    let target = target
        .downcast_ref::<T>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T>()));

    target
}

fn debug_props<P>(props: &dyn Any) -> &dyn fmt::Debug
where
    P: Props,
{
    let props = props
        .downcast_ref::<P>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<P>()));

    props
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
