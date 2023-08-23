use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_extras::InternalEntityKey;

pub(crate) struct TemporaryEntityKeyStorage {
    pub(crate) used_holes: usize,
    pub(crate) tail_allocations: usize,
}

impl TemporaryEntityKeyStorage {
    pub(crate) fn new() -> TemporaryEntityKeyStorage {
        TemporaryEntityKeyStorage {
            used_holes: 0,
            tail_allocations: 0,
        }
    }
}

impl TemporaryEntityKeyStorage {
    pub(crate) fn next_entity_key(&mut self, _entity_storage: &EntityStorage) -> InternalEntityKey {
        todo!()
    }
}
