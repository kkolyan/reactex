use crate::internal::entity_storage::EntityStorage;
use crate::internal::entity_storage::ValidateUncommitted;
use crate::internal::world_extras::InternalEntityKey;
use crate::world_result::EntityError;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct EntityKey {
    pub(crate) inner: InternalEntityKey,
}

impl Display for EntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Debug for EntityKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
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
