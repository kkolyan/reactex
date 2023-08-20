use crate::entity::Entity;
use crate::entity_key::EntityKey;
use crate::entity_uncommitted::UncommittedEntity;
use crate::filter::FilterDesc;
use crate::internal::entity_storage::ValidateUncommitted;
use crate::world_result::EntityError;
use crate::StableWorld;
use crate::VolatileWorld;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Copy, Clone)]
pub struct Ctx<'a, TSignal = ()> {
    pub signal: &'a TSignal,
    pub(crate) stable: &'a StableWorld,
    pub(crate) volatile: &'a RefCell<&'a mut VolatileWorld>,
}

impl<'a, TSignal> Ctx<'a, TSignal> {
    pub fn new(
        signal: &'a TSignal,
        stable: &'a StableWorld,
        volatile: &'a RefCell<&'a mut VolatileWorld>,
    ) -> Ctx<'a, TSignal> {
        Ctx {
            signal,
            stable,
            volatile,
        }
    }

    pub fn create_entity<'b>(&'b self) -> UncommittedEntity<'a> {
        let entity_storage = &mut self.stable.entity_storage.borrow_mut();
        let volatile_world = &mut self.volatile.borrow_mut();
        let key = volatile_world
            .deref_mut()
            .create_entity(entity_storage.deref_mut());
        UncommittedEntity {
            key,
            stable: self.stable,
            volatile: self.volatile,
        }
    }

    pub fn get_entity<'b>(&'b self, key: EntityKey) -> Option<Entity<'a>> {
        let result = key.validate(
            self.stable.entity_storage.borrow().deref(),
            ValidateUncommitted::DenyUncommitted,
        );
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
            volatile: self.volatile,
        })
    }

    pub fn send_signal<T: 'static>(&self, signal: T) {
        let volatile = &mut self.volatile.borrow_mut();
        volatile.signal(signal);
    }

    pub fn query(&self, filter: FilterDesc, callback: impl FnMut(EntityKey)) {
        self.stable.query(filter, callback);
    }
}
