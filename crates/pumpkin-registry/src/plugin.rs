use std::any::TypeId;

use crate::{Identifier, builder::RegistryBuilderShard};

pub struct PluginHolder {
    pub plugins: Vec<Box<dyn RegistryEventListener>>,
}

// each plugin has to implement something like this.
pub trait RegistryEventListener {
    /// How many entries this plugin plans to register for a given registry.
    /// Acquiring this information in advance means the registry's hashmap/vec never need resizing.
    fn registry_count(&self, registry: &Identifier, type_id: TypeId) -> usize;

    /// A [`TypedRegistryBuilderShard`](crate::builder::TypedRegistryBuilderShard)
    /// containing all values/identifiers this plugin intends on registering for
    /// the given registry.
    ///
    /// This method will not be called if [`Self::registry_count`] previously returned 0.
    fn registry(
        &self,
        registry: &Identifier,
        type_id: TypeId,
        min_network_id: usize,
    ) -> Box<dyn RegistryBuilderShard>;
}
