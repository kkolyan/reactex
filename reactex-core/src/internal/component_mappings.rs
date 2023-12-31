use crate::component::ComponentType;
use crate::internal::component_pool_manager::ComponentDataKey;
use crate::internal::world_extras::EntityIndex;
use std::collections::HashMap;

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
