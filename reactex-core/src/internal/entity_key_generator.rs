use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_extras::InternalEntityKey;

pub(crate) struct TemporaryEntityKeyStorage {}

impl TemporaryEntityKeyStorage {
    pub(crate) fn new() -> TemporaryEntityKeyStorage {
        todo!()
    }
}

impl TemporaryEntityKeyStorage {
    pub(crate) fn next_entity_key(&mut self, _entity_storage: &EntityStorage) -> InternalEntityKey {
        todo!()
    }
}
