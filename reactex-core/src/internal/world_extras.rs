use crate::entity_key::EntityKey;
use crate::internal::cause::Cause;
use crate::internal::component_pool_manager::TempComponentDataKey;
use crate::internal::signal_storage::SignalDataKey;
use crate::utils::opt_tiny_vec::OptTinyVec;
use crate::Ctx;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Default)]
pub(crate) struct DeleteQueue<TKey> {
    pub(crate) before_disappear: HashMap<TKey, OptTinyVec<Cause>>,
    pub(crate) after_disappear: HashMap<TKey, OptTinyVec<Cause>>,
}

impl<TKey> DeleteQueue<TKey> {
    pub(crate) fn new() -> DeleteQueue<TKey> {
        DeleteQueue {
            before_disappear: HashMap::new(),
            after_disappear: HashMap::new(),
        }
    }
}

pub(crate) struct ComponentAdd {
    pub(crate) data: TempComponentDataKey,
    pub(crate) cause: Cause,
}

pub(crate) struct EventHandler {
    pub(crate) name: &'static str,
    pub(crate) callback: Box<dyn Fn(Ctx, EntityKey)>,
}

pub(crate) struct Signal {
    pub(crate) payload_type: TypeId,
    pub(crate) data_key: SignalDataKey,
    pub(crate) cause: Cause,
}

#[derive(Debug)]
pub(crate) enum ComponentEventType {
    Appear,
    Disappear,
}

pub(crate) struct ComponentModify {
    pub(crate) callback: Box<dyn FnOnce(&mut dyn Any)>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct InternalEntityKey {
    pub(crate) index: EntityIndex,
    pub(crate) generation: EntityGeneration,
    pub(crate) temp: bool,
}

impl Display for InternalEntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.index.index, self.generation.0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct EntityGeneration(u16);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct EntityIndex {
    pub(crate) index: u32,
}

impl Display for EntityIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:_", self.index)
    }
}

impl InternalEntityKey {
    pub fn export(&self) -> EntityKey {
        let index = self.index;
        let generation = self.generation;
        EntityKey {
            inner: InternalEntityKey {
                index,
                generation,
                temp: false,
            },
        }
    }
}

impl EntityGeneration {
    pub fn new() -> Self {
        EntityGeneration(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}
