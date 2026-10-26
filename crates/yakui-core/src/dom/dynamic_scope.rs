use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub struct DynamicScope {
    inner: RefCell<Inner>,
}

#[derive(Default)]
struct Inner {
    scopes: Vec<Scope>,
    stack: Vec<usize>,
}

#[derive(Clone, Default)]
pub struct Scope {
    storage: HashMap<TypeId, Rc<dyn Any>>,
}

impl DynamicScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.scopes.clear();
        inner.stack.clear();
    }

    pub fn get<T: 'static>(&self, scope: usize) -> Option<Rc<T>> {
        let inner = self.inner.borrow();

        let scope = inner.scopes.get(scope)?;
        let item = scope.storage.get(&TypeId::of::<T>())?;
        Some(Rc::downcast(item.clone()).unwrap())
    }

    pub fn current_scope(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        inner.stack.last().copied()
    }

    pub fn push_scope(&self) -> usize {
        let mut inner = self.inner.borrow_mut();
        let mut new_scope = Scope::default();

        if let Some(parent) = inner.stack.last().and_then(|&i| inner.scopes.get(i)) {
            new_scope = parent.clone();
        }

        let index = inner.scopes.len();
        inner.scopes.push(new_scope);
        inner.stack.push(index);

        index
    }

    #[track_caller]
    pub fn pop_scope(&self) {
        let mut inner = self.inner.borrow_mut();

        assert!(
            inner.stack.pop().is_some(),
            "cannot pop_scope when the stack is empty"
        );
    }

    pub fn write_item<T: 'static>(&self, value: T) {
        let mut inner = self.inner.borrow_mut();
        let inner = &mut *inner;

        let scope = inner
            .stack
            .last()
            .and_then(|&i| inner.scopes.get_mut(i))
            .unwrap_or_else(|| panic!("cannot write_item when the stack is empty"));

        scope.storage.insert(TypeId::of::<T>(), Rc::new(value));
    }
}
