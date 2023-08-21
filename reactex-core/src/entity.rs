use std::cell::RefCell;

use crate::component::EcsComponent;
use crate::entity_key::EntityKey;
use crate::internal::change_buffer::Change;
use crate::internal::change_buffer::ChangeBuffer;
use crate::internal::component_key::ComponentKey;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_extras::InternalEntityKey;
use crate::StableWorld;

#[derive(Copy, Clone)]
pub struct Entity<'a> {
    pub(crate) key: InternalEntityKey,
    pub(crate) stable: &'a StableWorld,
    pub(crate) entity_storage: &'a EntityStorage,
    pub(crate) changes: &'a RefCell<&'a mut ChangeBuffer>,
}

impl<'a> Entity<'a> {
    pub fn key(self) -> EntityKey {
        self.key.export()
    }

    pub fn destroy(self) {
        let mut changes = self.changes.borrow_mut();
        changes.changes.push(Change::EntityDestroy(self.key));
    }

    pub fn add<TComponent: EcsComponent>(&self, value: TComponent) {
        let mut changes = self.changes.borrow_mut();
        changes.changes.push(Change::ComponentAdd(
            ComponentKey::new(self.key, TComponent::get_component_type()),
            Box::new(value),
        ));
    }

    pub fn get<TComponent: EcsComponent>(&self) -> Option<&TComponent> {
        self.stable
            .get_component::<TComponent>(self.key.export(), self.entity_storage)
            .unwrap()
    }

    pub fn remove<TComponent: EcsComponent>(&self) {
        let mut changes = self.changes.borrow_mut();
        changes
            .changes
            .push(Change::ComponentRemove(ComponentKey::new(
                self.key,
                TComponent::get_component_type(),
            )));
    }

    pub fn modify<TComponent: EcsComponent>(&self, change: impl FnOnce(&mut TComponent) + 'static) {
        let mut changes = self.changes.borrow_mut();
        changes.changes.push(Change::ComponentModification(
            ComponentKey::new(self.key, TComponent::get_component_type()),
            Box::new(|value| {
                let value = value.downcast_mut::<TComponent>().unwrap();
                change(value)
            }),
        ));
    }
}
