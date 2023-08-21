use crate::component::EcsComponent;
use crate::entity_key::EntityKey;
use crate::filter::FilterDesc;
use crate::internal::world_configure::ConfigurableWorld;
use crate::internal::world_core::World;
use crate::internal::world_pipeline::execute_all_internal;
use crate::world_result::WorldResult;
use crate::Ctx;

impl ConfigurableWorld {
    // I'm just too lazy to rewrite all tests to user API
    pub fn create_for_test() -> ConfigurableWorld {
        ConfigurableWorld::new()
    }

    pub(crate) fn new() -> Self {
        Self {
            fetus: World::new(),
        }
    }

    pub fn seal(self) -> World {
        self.fetus
    }

    pub fn add_global_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        callback: impl Fn(Ctx<T>) + 'static,
    ) {
        self.fetus.add_global_signal_handler(name, callback)
    }

    pub fn add_entity_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        filter: FilterDesc,
        callback: impl Fn(Ctx<T>, EntityKey) + 'static,
    ) {
        self.fetus.add_entity_signal_handler(name, filter, callback)
    }

    pub fn add_disappear_handler(
        &mut self,
        name: &'static str,
        filter_key: FilterDesc,
        callback: impl Fn(Ctx, EntityKey) + 'static,
    ) {
        self.fetus.add_disappear_handler(name, filter_key, callback)
    }

    pub fn add_appear_handler(
        &mut self,
        name: &'static str,
        filter_key: FilterDesc,
        callback: impl Fn(Ctx, EntityKey) + 'static,
    ) {
        self.fetus.add_appear_handler(name, filter_key, callback)
    }
}

// control
impl World {
    pub fn signal<T: 'static>(&mut self, payload: T) {
        self.volatile.signal(payload)
    }

    pub fn execute_all(&mut self) {
        execute_all_internal(self);
    }
}

// work with entities
impl World {
    pub fn create_entity(&mut self) -> EntityKey {
        let entity_storage = &mut self.entity_storage;
        self.volatile.create_entity(entity_storage).export()
    }

    pub fn destroy_entity(&mut self, entity: EntityKey) -> WorldResult {
        let entity_storage = &mut self.entity_storage;
        self.volatile.destroy_entity(entity, entity_storage)
    }

    pub fn entity_exists(&self, entity: EntityKey) -> bool {
        self.stable.entity_exists(entity, &self.entity_storage)
    }

    pub fn query(&mut self, filter: FilterDesc, callback: impl FnMut(EntityKey)) {
        self.stable.query(filter, callback)
    }
}

// work with components
impl World {
    pub fn get_component<T: EcsComponent>(&self, entity: EntityKey) -> WorldResult<Option<&T>> {
        self.stable.get_component::<T>(entity, &self.entity_storage)
    }
    pub fn has_component<T: EcsComponent>(&self, entity: EntityKey) -> WorldResult<bool> {
        self.stable.has_component::<T>(entity, &self.entity_storage)
    }

    pub fn modify_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        change: impl FnOnce(&mut T) + 'static,
    ) -> WorldResult {
        let entity_storage = &self.entity_storage;
        self.volatile
            .modify_component(entity, change, entity_storage)
    }

    pub fn add_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        component: T,
    ) -> WorldResult {
        let entity_storage = &self.entity_storage;
        self.volatile
            .add_component(entity, component, entity_storage)
    }

    pub fn remove_component<T: EcsComponent>(&mut self, entity: EntityKey) -> WorldResult {
        let entity_storage = &self.entity_storage;
        self.volatile
            .remove_component::<T>(entity, entity_storage, &self.stable.component_mappings)
    }
}
