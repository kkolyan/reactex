use crate::component::EcsComponent;
use crate::entity::Entity;
use crate::entity_key::EntityKey;
use crate::internal::world_extras::InternalEntityKey;
use crate::StableWorld;
use crate::VolatileWorld;
use std::cell::RefCell;

#[derive(Copy, Clone)]
pub struct UncommittedEntity<'a> {
    pub(crate) key: InternalEntityKey,
    pub(crate) stable: &'a StableWorld,
    pub(crate) volatile: &'a RefCell<&'a mut VolatileWorld>,
}

impl<'a> UncommittedEntity<'a> {
    pub fn key(&self) -> EntityKey {
        self.key.export()
    }

    pub fn destroy(self) {
        Entity {
            key: self.key,
            stable: self.stable,
            volatile: self.volatile,
        }
        .destroy();
    }

    pub fn add<TComponent: EcsComponent>(&self, value: TComponent) {
        Entity {
            key: self.key,
            stable: self.stable,
            volatile: self.volatile,
        }
        .add(value);
    }
}
