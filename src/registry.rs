use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::component::{Component, ComponentImpl};

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
            .or_insert_with(ComponentImpl::new::<T>);
    }
}

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry").finish()
    }
}
