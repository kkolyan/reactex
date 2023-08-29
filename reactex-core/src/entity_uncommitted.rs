use std::cell::RefCell;
use std::ops::Add;

use crate::component::EcsComponent;
use crate::entity::Entity;
use crate::entity_key::EntityKey;
use crate::internal::change_buffer::ChangeBuffer;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_extras::InternalEntityKey;
use crate::StableWorld;

#[derive(Copy, Clone)]
pub struct UncommittedEntity<'a> {
    pub(crate) key: InternalEntityKey,
    pub(crate) stable: &'a StableWorld,
    pub(crate) entity_storage: &'a EntityStorage,
    pub(crate) changes: &'a RefCell<&'a mut ChangeBuffer>,
}

impl<'a> UncommittedEntity<'a> {
    pub fn key(&self) -> EntityKey {
        self.key.export()
    }

    pub fn destroy(self) {
        Entity {
            key: self.key,
            stable: self.stable,
            entity_storage: self.entity_storage,
            changes: self.changes,
        }
        .destroy();
    }

    pub fn add<TComponent: EcsComponent>(self, value: TComponent) -> UncommittedEntity<'a> {
        Entity {
            key: self.key,
            stable: self.stable,
            entity_storage: self.entity_storage,
            changes: self.changes,
        }
        .add(value);
        self
    }
}

impl<'a, TComponent: EcsComponent> Add<TComponent> for UncommittedEntity<'a> {
    type Output = Self;

    fn add(self, rhs: TComponent) -> Self::Output {
        UncommittedEntity::add(self, rhs)
    }
}
