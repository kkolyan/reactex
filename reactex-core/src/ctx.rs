use crate::entity::Entity;
use crate::entity_key::EntityKey;
use crate::entity_uncommitted::UncommittedEntity;
use crate::filter::FilterDesc;
use crate::internal::change_buffer::Change;
use crate::internal::change_buffer::ChangeBuffer;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::entity_storage::ValidateUncommitted;
use crate::world_result::EntityError;
use crate::StableWorld;
use std::cell::RefCell;

#[derive(Copy, Clone)]
pub struct Ctx<'a, TSignal = ()> {
    pub signal: &'a TSignal,
    stable: &'a StableWorld,
    entity_storage: &'a EntityStorage,
    changes: &'a RefCell<&'a mut ChangeBuffer>,
}

impl<'a, TSignal> Ctx<'a, TSignal> {
    pub(crate) fn new(
        signal: &'a TSignal,
        stable: &'a StableWorld,
        entity_storage: &'a EntityStorage,
        changes: &'a RefCell<&'a mut ChangeBuffer>,
    ) -> Ctx<'a, TSignal> {
        Ctx {
            signal,
            stable,
            entity_storage,
            changes,
        }
    }

    pub fn create_entity<'b>(&'b self) -> UncommittedEntity<'a> {
        let mut changes = self.changes.borrow_mut();
        let entity_key = self
            .entity_storage
            .generate_temporary(&mut changes.entity_key_generator);
        let key = entity_key.inner;
        changes
            .entity_key_generator
            .next_entity_key(&self.entity_storage);
        changes.changes.push(Change::EntityCreate(entity_key));
        UncommittedEntity {
            key,
            stable: self.stable,
            entity_storage: self.entity_storage,
            changes: self.changes,
        }
    }

    pub fn get_entity<'b>(&'b self, key: EntityKey) -> Option<Entity<'a>> {
        let result = key.validate(&self.entity_storage, ValidateUncommitted::DenyUncommitted);
        let entity_key = match result {
            Ok(it) => Some(it),
            Err(err) => match err {
                EntityError::NotExists => None,
                EntityError::NotCommitted => {
                    panic!("attempt to transform UncommittedEntity to Entity detected")
                }
                EntityError::IsStale => None,
            },
        }?;
        Some(Entity {
            key: entity_key,
            stable: self.stable,
            entity_storage: self.entity_storage,
            changes: self.changes,
        })
    }

    pub fn send_signal<T: 'static>(&self, signal: T) {
        let mut changes = self.changes.borrow_mut();
        changes
            .changes
            .push(Change::SignalSent(Box::new(|volatile| {
                volatile.signal(signal);
            })));
    }

    pub fn query(&self, filter: FilterDesc, callback: impl FnMut(EntityKey)) {
        self.stable.query(filter, callback);
    }
}
