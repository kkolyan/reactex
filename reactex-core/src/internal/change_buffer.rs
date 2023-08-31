use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use log::trace;

use crate::internal::component_key::ComponentKey;
use crate::internal::component_mappings::ComponentMappingStorage;
use crate::internal::entity_key_generator::TemporaryEntityKeyStorage;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_extras::InternalEntityKey;

use crate::VolatileWorld;

pub(crate) enum Change {
    EntityCreate(TempEntityKey),
    EntityDestroy(InternalEntityKey),
    ComponentAdd(ComponentKey, Box<dyn Any>),
    ComponentRemove(ComponentKey),
    ComponentModification(ComponentKey, ComponentModification),
    SignalSend(Box<dyn FnOnce(&mut VolatileWorld)>, &'static str),
}

type ComponentModification = Box<dyn FnOnce(&mut dyn Any)>;

pub(crate) struct TempEntityKey {
    pub(crate) inner: InternalEntityKey,
}

impl Display for TempEntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Debug for TempEntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
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
                    trace!("request create entity {}", entity);
                    volatile.persist_entity(entity, entity_storage);
                }
                Change::EntityDestroy(entity) => {
                    trace!("request destroy entity {}", entity);
                    volatile.destroy_entity_internal(entity, entity_storage);
                }
                Change::ComponentAdd(component_key, value) => {
                    trace!("request add component {}", component_key);
                    volatile.add_component_dyn_internal(component_key, value);
                }
                Change::ComponentRemove(component_key) => {
                    trace!("request remove component {}", component_key);
                    volatile
                        .remove_component_internal(component_key, component_mappings)
                        .unwrap()
                },
                Change::ComponentModification(component_key, modification) => {
                    trace!("request modify component {}", component_key);
                    volatile.modify_component_internal(component_key, modification);
                }
                Change::SignalSend(signal, type_name) => {
                    trace!("request signal send {}", type_name);
                    signal(volatile);
                }
            }
        }
    }
}
