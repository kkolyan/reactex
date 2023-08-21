use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::internal::component_key::ComponentKey;
use crate::internal::component_mappings::ComponentMappingStorage;
use crate::internal::entity_key_generator::TemporaryEntityKeyStorage;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_extras::InternalEntityKey;
use crate::StableWorld;
use crate::VolatileWorld;

pub(crate) enum Change {
    EntityCreate(TempEntityKey),
    EntityDestroy(InternalEntityKey),
    ComponentAdd(ComponentKey, Box<dyn Any>),
    ComponentRemove(ComponentKey),
    ComponentModification(ComponentKey, ComponentModification),
    SignalSent(Box<dyn FnOnce(&mut VolatileWorld)>),
}

type ComponentModification = Box<dyn FnOnce(&mut dyn Any)>;

#[derive(Debug)]
pub(crate) struct TempEntityKey {
    pub(crate) inner: InternalEntityKey,
}
pub(crate) struct ChangeBuffer {
    pub(crate) changes: Vec<Change>,
    pub(crate) entity_key_generator: TemporaryEntityKeyStorage,
}

impl ChangeBuffer {
    pub(crate) fn new(entity_key_generator: TemporaryEntityKeyStorage) -> Self {
        Self {
            changes: vec![],
            entity_key_generator,
        }
    }

    pub(crate) fn apply_to(
        self,
        volatile: &mut VolatileWorld,
        entity_storage: &mut EntityStorage,
        component_mappings: &ComponentMappingStorage,
    ) {
        for change in self.changes {
            match change {
                Change::EntityCreate(entity) => {
                    volatile.persist_entity(entity, entity_storage);
                }
                Change::EntityDestroy(entity) => {
                    volatile.destroy_entity_internal(entity, entity_storage);
                }
                Change::ComponentAdd(component_key, value) => {
                    volatile.add_component_dyn_internal(component_key, value);
                }
                Change::ComponentRemove(component_key) => volatile
                    .remove_component_internal(component_key, component_mappings)
                    .unwrap(),
                Change::ComponentModification(component_key, modification) => {
                    volatile.modify_component_internal(component_key, modification);
                }
                Change::SignalSent(signal) => {
                    signal(volatile);
                }
            }
        }
    }
}
