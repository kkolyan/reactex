use crate::component::ComponentType;
use crate::entity::EntityIndex;
use std::collections::HashMap;
use crate::world_mod::component_pool_manager::{ComponentDataKey};

#[derive(Default)]
pub struct ComponentMappingStorage {
    pub(crate) data_by_entity_by_type:
        HashMap<ComponentType, HashMap<EntityIndex, ComponentDataKey>>,
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
