use std::any::{Any, TypeId};

use rustc_hash::FxHashMap;

use crate::{
    Identifier, plugin::PluginHolder, reloadable::ReloadableRegistry, r#static::StaticRegistry,
};

pub struct TypedRegistryBuilderShard<T: Any> {
    identifiers: Vec<Identifier>,
    values: Vec<T>,
}

pub trait RegistryBuilderShard: Any {}
impl<T: Any> RegistryBuilderShard for TypedRegistryBuilderShard<T> {}

pub struct RegistryBuilder<T: Send + Sync + 'static> {
    entries: Vec<T>,
    mapping: FxHashMap<Identifier, usize>,
}

impl<T: Send + Sync + 'static> RegistryBuilder<T> {
    /// Build a registry where Pumpkin's internal entries do not need to be copied to the heap, \
    /// collecting additional entries from plugins.
    #[must_use]
    pub fn r#static(
        name: &Identifier,
        static_entries: &'static [T],
        identifiers: &[Identifier],
        plugin_holder: &PluginHolder,
    ) -> StaticRegistry<T> {
        let statics = static_entries.len();
        assert!(statics == identifiers.len());

        let mapping: FxHashMap<_, _> = identifiers
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        let Self { entries, mapping } =
            Self::new(statics, Vec::new(), mapping, name, plugin_holder);

        StaticRegistry::new(static_entries, entries.into_boxed_slice(), mapping)
    }

    /// Build a registry where all data lives on the heap, \
    /// collecting additional entries from plugins.
    ///
    /// These registries may not be reloaded.
    #[must_use]
    pub fn frozen(
        name: &Identifier,
        internal_entries: Vec<T>,
        identifiers: Vec<Identifier>,
        plugin_holder: &PluginHolder,
    ) -> ReloadableRegistry<T> {
        assert!(internal_entries.len() == identifiers.len());
        let mapping: FxHashMap<_, _> = identifiers
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        let Self { entries, mapping } =
            Self::new(0, internal_entries, mapping, name, plugin_holder);

        ReloadableRegistry::new(entries.into_boxed_slice(), mapping)
    }

    /// Build a reloadable registry, \
    /// collecting additional entries from plugins.
    #[must_use]
    pub fn reloadable(name: &Identifier, plugin_holder: &PluginHolder) -> ReloadableRegistry<T> {
        let Self { entries, mapping } =
            Self::new(0, Vec::new(), FxHashMap::default(), name, plugin_holder);

        ReloadableRegistry::new(entries.into_boxed_slice(), mapping)
    }

    #[expect(clippy::expect_used, clippy::panic, reason = "it's a code example")]
    fn new(
        offset: usize,
        mut entries: Vec<T>,
        mut mapping: FxHashMap<Identifier, usize>,
        name: &Identifier,
        plugin_holder: &PluginHolder,
    ) -> Self {
        let type_id = TypeId::of::<T>();

        let counts: Box<_> = plugin_holder
            .plugins
            .iter()
            .map(|plugin| plugin.registry_count(name, type_id))
            .collect();
        let total = counts.iter().sum();

        entries.reserve_exact(total);
        mapping.reserve(total);

        let mut i = offset + entries.len();
        for (plugin, count) in plugin_holder.plugins.iter().zip(counts) {
            let shard = plugin.registry(name, type_id, i);
            let shard = (shard as Box<dyn Any>)
                .downcast::<TypedRegistryBuilderShard<T>>()
                .expect("Type mismatch");
            assert!(shard.identifiers.len() == count && shard.values.len() == count);

            entries.extend(shard.values);
            for identifier in shard.identifiers {
                let opt = mapping.insert(identifier.clone(), i);
                if let Some(id) = opt {
                    panic!(
                        "Tried to re-register identifier `{identifier}` for Registry `{name}`, but it is already bound to network_id `{id}`"
                    );
                }

                i += 1;
            }
        }

        Self { entries, mapping }
    }
}
