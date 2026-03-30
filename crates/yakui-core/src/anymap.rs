//! We have anymap at home
use std::{
    any::{Any, TypeId},
    collections::{
        hash_map::{OccupiedEntry, VacantEntry},
        HashMap,
    },
};

pub struct AnyMap(HashMap<TypeId, Box<dyn Any>>);

impl AnyMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn entry<T: 'static>(&'_ mut self) -> Entry<'_> {
        match self.0.entry(TypeId::of::<T>()) {
            std::collections::hash_map::Entry::Vacant(v) => Entry::Vacant(v),
            std::collections::hash_map::Entry::Occupied(v) => Entry::Occupied(v),
        }
    }
}

pub enum Entry<'a> {
    Vacant(VacantEntry<'a, TypeId, Box<dyn Any>>),
    Occupied(OccupiedEntry<'a, TypeId, Box<dyn Any>>),
}

impl<'a> Entry<'a> {
    pub fn or_insert_with<T: 'static>(self, f: impl FnOnce() -> T) -> &'a mut T {
        match self {
            Entry::Vacant(v) => v.insert(Box::new(f())).downcast_mut::<T>().unwrap(),
            Entry::Occupied(o) => o.into_mut().downcast_mut::<T>().unwrap(),
        }
    }
}
