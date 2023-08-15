use crate::component::StaticComponentType;
use crate::entity::EntityKey;
use crate::filter::filter_desc::FilterDesc;
use crate::world_mod::world::World;
use crate::world_mod::world::WorldResult;

impl World {
    pub fn modify_component<T: StaticComponentType>(
        &mut self,
        entity: EntityKey,
        change: impl FnOnce(&mut T) + 'static,
    ) -> WorldResult {
        let entity_storage = self.stable.entity_storage.get_mut();
        self.volatile
            .modify_component(entity, change, entity_storage)
    }

    pub fn add_component<T: StaticComponentType>(
        &mut self,
        entity: EntityKey,
        component: T,
    ) -> WorldResult {
        let entity_storage = self.stable.entity_storage.get_mut();
        self.volatile
            .add_component(entity, component, entity_storage)
    }

    pub fn remove_component<T: StaticComponentType>(&mut self, entity: EntityKey) -> WorldResult {
        let entity_storage = self.stable.entity_storage.get_mut();
        self.volatile.remove_component::<T>(entity, entity_storage)
    }
    pub fn create_entity(&mut self) -> EntityKey {
        let entity_storage = self.stable.entity_storage.get_mut();
        self.volatile.create_entity(entity_storage)
    }

    pub fn destroy_entity(&mut self, entity: EntityKey) -> WorldResult {
        let entity_storage = self.stable.entity_storage.get_mut();
        self.volatile.destroy_entity(entity, entity_storage)
    }
    pub fn get_component<T: StaticComponentType>(
        &self,
        entity: EntityKey,
    ) -> WorldResult<Option<&T>> {
        self.stable.get_component::<T>(entity)
    }
    pub fn has_component<T: StaticComponentType>(&self, entity: EntityKey) -> WorldResult<bool> {
        self.stable.has_component::<T>(entity)
    }

    pub fn entity_exists(&self, entity: EntityKey) -> bool {
        self.stable.entity_exists(entity)
    }
    pub fn query(&self, filter: FilterDesc, callback: impl FnMut(EntityKey)) {
        self.stable.query(filter, callback)
    }

    pub fn signal<T: 'static>(&mut self, payload: T) {
        self.volatile.signal(payload)
    }
}
