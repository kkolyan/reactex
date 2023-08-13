use std::collections::HashMap;
use crate::component::ComponentType;
use crate::entity::EntityIndex;
use crate::pools::PoolKey;

#[derive(Default)]
pub struct ComponentMappingStorage {
    pub(crate) data_by_entity_by_type: HashMap<ComponentType, HashMap<EntityIndex, ComponentDataKey>>,
}

impl ComponentMappingStorage {
    pub(crate) fn has_component_no_validation(
        &self,
        entity: EntityIndex,
        component_type: ComponentType,
    ) -> bool {
        self.data_by_entity_by_type
            .get(&component_type)
            .map(|it| it.contains_key(&entity))
            .unwrap_or(false)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentDataKey {
    pub index: usize,
}

impl PoolKey for ComponentDataKey {
    fn as_usize(&self) -> usize {
        self.index
    }
    fn from_usize(value: usize) -> Self {
        ComponentDataKey { index: value }
    }
}
