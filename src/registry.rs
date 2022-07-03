use std::any::{type_name, Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::component::Component;

#[derive(Clone, Copy)]
pub struct ComponentImpl {
    pub new: fn(&dyn Any) -> Box<dyn Any>,
    pub update: fn(&mut dyn Any, &dyn Any),

    pub debug: fn(&dyn Any) -> &dyn fmt::Debug,
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

    pub fn register<T, P>(&self)
    where
        T: Component<P>,
        P: 'static,
    {
        self.inner
            .borrow_mut()
            .types
            .entry(TypeId::of::<T>())
            .or_insert(ComponentImpl {
                new: new::<T, P>,
                update: update::<T, P>,
                debug: debug::<T, P>,
            });
    }
}

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry").finish()
    }
}

fn new<T, P>(props: &dyn Any) -> Box<dyn Any>
where
    T: Component<P>,
    P: 'static,
{
    let props = props.downcast_ref::<P>().unwrap_or_else(|| {
        panic!(
            "Component {} expects props of type {} (ID {:?}), got ID {:?}",
            type_name::<T>(),
            type_name::<P>(),
            TypeId::of::<P>(),
            props.type_id(),
        )
    });

    Box::new(T::new(props))
}

fn update<T, P>(target: &mut dyn Any, props: &dyn Any)
where
    T: Component<P>,
    P: 'static,
{
    let target = target
        .downcast_mut::<T>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T>()));

    let props = props.downcast_ref::<P>().unwrap_or_else(|| {
        panic!(
            "Component {} expects props of type {}",
            type_name::<T>(),
            type_name::<P>()
        )
    });

    T::update(target, props);
}

fn debug<T, P>(target: &dyn Any) -> &dyn fmt::Debug
where
    T: Component<P>,
    P: 'static,
{
    let target = target
        .downcast_ref::<T>()
        .unwrap_or_else(|| panic!("Type mixup: unexpected {}", type_name::<T>()));

    target
}
