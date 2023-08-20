use crate::component::EcsComponent;
use crate::entity_key::EntityKey;
use crate::internal::world_extras::InternalEntityKey;
use crate::StableWorld;
use crate::VolatileWorld;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Copy, Clone)]
pub struct Entity<'a> {
    pub(crate) key: InternalEntityKey,
    pub(crate) stable: &'a StableWorld,
    pub(crate) volatile: &'a RefCell<&'a mut VolatileWorld>,
}

impl<'a> Entity<'a> {
    pub fn key(self) -> EntityKey {
        self.key.export()
    }

    pub fn destroy(self) {
        let entity_storage = &mut self.stable.entity_storage.borrow_mut();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .destroy_entity(self.key.export(), entity_storage.deref_mut())
            .unwrap();
    }

    pub fn add<TComponent: EcsComponent>(&self, value: TComponent) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .add_component(self.key.export(), value, entity_storage.deref())
            .unwrap();
    }

    pub fn get<TComponent: EcsComponent>(&self) -> Option<&TComponent> {
        self.stable
            .get_component::<TComponent>(self.key.export())
            .unwrap()
    }

    pub fn remove<TComponent: EcsComponent>(&self) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .remove_component::<TComponent>(
                self.key.export(),
                entity_storage.deref(),
                &self.stable.component_mappings,
            )
            .unwrap()
    }

    pub fn modify<TComponent: EcsComponent>(&self, change: impl FnOnce(&mut TComponent) + 'static) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .modify_component::<TComponent>(self.key.export(), change, entity_storage.deref())
            .unwrap()
    }
}
