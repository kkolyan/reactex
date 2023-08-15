use crate::component::StaticComponentType;
use crate::entity::EntityIndex;
use crate::entity::EntityKey;
use crate::pools::SpecificPool;
use crate::world_mod::component_pool_manager::ComponentDataKey;
use crate::world_mod::entity_storage::ValidateUncommitted::AllowUncommitted;
use crate::world_mod::entity_storage::ValidateUncommitted::DenyUncommitted;
use crate::world_mod::world::StableWorld;
use crate::world_mod::world::WorldResult;
use log::trace;
use std::ops::Deref;

impl StableWorld {
    pub fn get_component<T: StaticComponentType>(
        &self,
        entity: EntityKey,
    ) -> WorldResult<Option<&T>> {
        let entity = entity
            .validate(self.entity_storage.borrow().deref(), DenyUncommitted)?
            .index;
        Ok(self.get_component_no_validation(entity))
    }

    fn get_component_no_validation<T: StaticComponentType>(
        &self,
        entity: EntityIndex,
    ) -> Option<&T> {
        let data = self
            .component_mappings
            .data_by_entity_by_type
            .get(&T::get_component_type())?
            .get(&entity)?;
        let instance = self.get_component_data::<T>()?.get(data);
        trace!("component found: {:?}", instance);
        instance
    }

    pub(crate) fn get_component_data<T: StaticComponentType>(
        &self,
    ) -> Option<&SpecificPool<ComponentDataKey, T>> {
        self.component_data
            .get_pool(T::get_component_type())?
            .specializable()
            .try_specialize::<T>()
    }
    pub fn has_component<T: StaticComponentType>(&self, entity: EntityKey) -> WorldResult<bool> {
        let entity = entity
            .validate(self.entity_storage.borrow().deref(), DenyUncommitted)?
            .index;
        Ok(self
            .component_mappings
            .data_by_entity_by_type
            .get(&T::get_component_type())
            .map(|it| it.contains_key(&entity))
            .unwrap_or(false))
    }

    pub fn entity_exists(&self, entity: EntityKey) -> bool {
        entity
            .validate(self.entity_storage.borrow().deref(), AllowUncommitted)
            .is_ok()
    }
}
