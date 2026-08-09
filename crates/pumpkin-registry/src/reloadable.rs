use std::{
    any::{TypeId, type_name},
    sync::RwLock,
};

use rustc_hash::FxHashMap;

use crate::{Identifier, Registry, TypedRegistry, frozen::FrozenRegistry};

/// A registry holding runtime-reloadable (e.g. datapack) data.
pub struct ReloadableRegistry<T: Send + Sync + 'static> {
    inner: RwLock<FrozenRegistry<T>>,
}

impl<T: Send + Sync + 'static> ReloadableRegistry<T> {
    pub(crate) const fn new(entries: Box<[T]>, mapping: FxHashMap<Identifier, usize>) -> Self {
        Self {
            inner: RwLock::new(FrozenRegistry::new(entries, mapping)),
        }
    }

    // some way to swap out the FrozenRegistry for datapack reloads
}

impl<T: Send + Sync + 'static> Registry for ReloadableRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

// std::sync::RwLockReadGuard does not have a map (or filter_map) fn in stable rust,
// making it (seemingly) impossible to return some form of mapped guard without unsafe code.
// This impl just clones for now.
impl<'a, T: Clone + Send + Sync + 'static> TypedRegistry<'a> for ReloadableRegistry<T> {
    type Item = T;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        self.inner.read().ok()?.by_id(id).cloned()
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.inner.read().ok()?.get_id(identifier)
    }
}
