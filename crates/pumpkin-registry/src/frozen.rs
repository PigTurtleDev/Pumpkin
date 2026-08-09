use std::any::{TypeId, type_name};

use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

use crate::{Registry, TypedRegistry};

/// An immutable registry holding heap-allocated data.
pub struct FrozenRegistry<T: Send + Sync + 'static> {
    entries: Box<[T]>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> FrozenRegistry<T> {
    pub(crate) const fn new(entries: Box<[T]>, mapping: FxHashMap<Identifier, usize>) -> Self {
        Self { entries, mapping }
    }
}

impl<T: Send + Sync + 'static> Registry for FrozenRegistry<T> {
    fn item_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn item_type_name(&self) -> &'static str {
        type_name::<T>()
    }
}

impl<'a, T: Send + Sync + 'static> TypedRegistry<'a> for FrozenRegistry<T> {
    type Item = &'a T;

    fn by_id(&'a self, id: usize) -> Option<Self::Item> {
        self.entries.get(id)
    }

    fn get_id(&self, identifier: &Identifier) -> Option<usize> {
        self.mapping.get(identifier).copied()
    }
}
