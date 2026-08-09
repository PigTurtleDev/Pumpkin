use std::any::{TypeId, type_name};

use rustc_hash::FxHashMap;

use crate::{Identifier, Registry, TypedRegistry};

/// An immutable registry holding 'static data.
pub struct StaticRegistry<T: Send + Sync + 'static> {
    static_entries: &'static [T],
    entries: Box<[T]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> StaticRegistry<T> {
    pub(crate) const fn new(
        static_entries: &'static [T],
        entries: Box<[T]>,
        mapping: FxHashMap<Identifier, usize>,
    ) -> Self {
        Self {
            static_entries,
            entries,
            mapping,
        }
    }
}

impl<T: Send + Sync + 'static> Registry for StaticRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

impl<'a, T: Send + Sync + 'static> TypedRegistry<'a> for StaticRegistry<T> {
    type Item = &'a T;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        if id < self.static_entries.len() {
            Some(&self.static_entries[id])
        } else {
            self.entries.get(id - self.static_entries.len())
        }
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.mapping.get(identifier).copied()
    }
}
