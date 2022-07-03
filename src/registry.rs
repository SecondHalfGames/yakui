use std::any::{type_name, Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use glam::Vec2;
use thunderdome::Index;

use crate::component::{Component, Props};
use crate::dom::Dom;
use crate::Constraints;

#[derive(Clone, Copy)]
pub struct ComponentImpl {
    pub new: fn(index: Index, &dyn Any) -> Box<dyn Any>,
    pub update: fn(&mut dyn Any, &dyn Any),
    pub size: fn(&dyn Any, &Dom, Constraints) -> Vec2,

    pub debug: fn(&dyn Any) -> &dyn fmt::Debug,
    pub debug_props: fn(&dyn Any) -> &dyn fmt::Debug,
}

#[derive(Clone)]
pub struct Registry {
    inner: Rc<RefCell<RegistryInner>>,
}

struct RegistryInner {
    types: HashMap<TypeId, ComponentImpl>,
}

impl Registry {
    pub fn new() -> Self {
        let inner = Rc::new(RefCell::new(RegistryInner {
            types: HashMap::new(),
        }));
        Self { inner }
    }

    pub fn get_by_id(&self, type_id: TypeId) -> Option<ComponentImpl> {
        self.inner.borrow().types.get(&type_id).copied()
    }

    pub fn register<T>(&self)
    where
        T: Component,
    {
        self.inner
            .borrow_mut()
            .types
            .entry(TypeId::of::<T>())
            .or_insert(ComponentImpl {
                new: new::<T>,
                update: update::<T>,
                size: size::<T>,
                debug: debug::<T>,
                debug_props: debug_props::<T::Props>,
            });
    }
}

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry").finish()
    }
}

fn new<T>(index: Index, props: &dyn Any) -> Box<dyn Any>
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
    let boxed: Box<dyn Any> = Box::new(value);
    boxed
}

fn update<T>(target: &mut dyn Any, props: &dyn Any)
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

fn size<T>(target: &dyn Any, dom: &Dom, constraints: Constraints) -> Vec2
where
    T: Component,
{
    let target = target
        .downcast_ref::<T>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T>()));

    target.size(dom, constraints)
}

fn debug<T>(target: &dyn Any) -> &dyn fmt::Debug
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
