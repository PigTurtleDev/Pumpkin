use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use pumpkin_util::identifier::Identifier;

pub mod builder;
pub mod frozen;
pub mod plugin;
pub mod reloadable;
pub mod r#static;

pub trait Registry: Any + Send + Sync {
    fn arc_dyn(self) -> Arc<dyn Registry>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
    fn item_type_id(&self) -> TypeId;
    fn item_type_name(&self) -> &'static str;
}

pub trait TypedRegistry<'a>: Registry {
    type Item;

    fn get(&'a self, identifier: &Identifier) -> Option<Self::Item> {
        self.get_id(identifier).and_then(|id| self.by_id(id))
    }

    fn by_id(&'a self, id: usize) -> Option<Self::Item>;
    fn get_id(&'a self, identifier: &Identifier) -> Option<usize>;
}

#[cfg(test)]
mod tests {
    //! more like examples

    use std::sync::{Arc, OnceLock};

    use pumpkin_util::identifier::Identifier;

    use crate::{Registry, builder::RegistryBuilder, plugin, reloadable::ReloadableRegistry};

    pub static ROOT_REGISTRY: OnceLock<ReloadableRegistry<Arc<dyn Registry>>> = OnceLock::new();

    // no pumpkin-data dependency for my sanity
    #[allow(unused)]
    pub struct Block {
        pub name: &'static str,
    }

    #[test]
    #[expect(
        clippy::expect_used,
        reason = "We shouldn't recover if a native plugin tries something stupid"
    )]
    fn possible_server_startup() {
        // ... server startup ...
        // ... find and init plugins
        let plugin_holder = plugin::PluginHolder {
            plugins: Vec::new(),
        };

        let identifiers = vec![
            Identifier::vanilla_static("block"),
            Identifier::vanilla_static("worldgen"),
        ];

        let block = RegistryBuilder::r#static(
            &identifiers[0],
            &[
                Block { name: "air" },
                Block { name: "stone" },
                Block { name: "granite" },
            ],
            &[
                Identifier::vanilla_static("air"),
                Identifier::vanilla_static("stone"),
                Identifier::vanilla_static("granite"),
            ],
            &plugin_holder,
        )
        .arc_dyn(); // probably store in a static as a typed Arc.

        let worldgen = RegistryBuilder::<Arc<dyn Registry>>::frozen(
            &identifiers[1],
            Vec::new(),
            Vec::new(),
            &plugin_holder,
        )
        .arc_dyn(); // probably store in a static as a typed Arc.

        let root = RegistryBuilder::frozen(
            &Identifier::vanilla_static("root"),
            vec![block, worldgen],
            identifiers,
            &plugin_holder,
        );

        ROOT_REGISTRY
            .set(root)
            .ok()
            .expect("Someone thought it would be funny to preemptively init the registry")
    }
}
