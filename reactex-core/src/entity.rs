use std::fmt::{Debug, Display, Formatter};
use crate::world_mod::entity_storage::EntityStorage;
use crate::world_mod::entity_storage::ValidateUncommitted;
use crate::world_mod::world::EntityError;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EntityKey {
    inner: InternalEntityKey,
}

impl Display for EntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct InternalEntityKey {
    pub(crate) index: EntityIndex,
    pub(crate) generation: EntityGeneration,
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
            inner: InternalEntityKey { index, generation },
        }
    }
}

impl EntityKey {
    pub(crate) fn validate(
        &self,
        entity_storage: &EntityStorage,
        uncommitted: ValidateUncommitted,
    ) -> Result<InternalEntityKey, EntityError> {
        entity_storage.validate(self.inner, uncommitted)?;
        Ok(self.inner)
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
