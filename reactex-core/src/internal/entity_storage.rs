use crate::internal::change_buffer::TempEntityKey;
use crate::internal::entity_key_generator::TemporaryEntityKeyStorage;
use crate::internal::entity_storage::ValidateUncommitted::DenyUncommitted;
use crate::internal::world_extras::EntityGeneration;
use crate::internal::world_extras::EntityIndex;
use crate::internal::world_extras::InternalEntityKey;
use crate::world_result::EntityError;
use crate::world_result::EntityError::IsStale;
use crate::world_result::EntityError::NotCommitted;
use crate::world_result::EntityError::NotExists;
use log::trace;
use std::mem;
use std::ops::Not;

pub(crate) struct EntityStorage {
    entities: Box<[EntityBox]>,
    allocation_boundary: usize,
    holes: Vec<usize>,
}

impl EntityStorage {
    pub(crate) fn generate_temporary(&self, _ctx: &mut TemporaryEntityKeyStorage) -> TempEntityKey {
        todo!()
    }

    pub(crate) fn persist_generated(&mut self, _entity: TempEntityKey) -> InternalEntityKey {
        todo!()
    }
}

impl EntityStorage {
    pub(crate) fn with_capacity(initial_capacity: usize) -> EntityStorage {
        EntityStorage {
            entities: vec![EntityBox::new(); initial_capacity].into_boxed_slice(),
            allocation_boundary: 0,
            holes: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
struct EntityBox {
    exists: bool,
    generation: EntityGeneration,
    committed: bool,
}

impl EntityBox {
    fn new() -> Self {
        EntityBox {
            exists: false,
            generation: EntityGeneration::new(),
            committed: false,
        }
    }
}

#[derive(Eq, PartialEq)]
pub(crate) enum ValidateUncommitted {
    AllowUncommitted,
    DenyUncommitted,
}

impl EntityStorage {
    #[inline(always)]
    pub(crate) fn validate(
        &self,
        entity: InternalEntityKey,
        uncommitted_strategy: ValidateUncommitted,
    ) -> Result<(), EntityError> {
        let found = self
            .entities
            .get(entity.index.index as usize)
            .expect("entity out of bounds");
        if !found.exists {
            return Err(NotExists);
        }
        if uncommitted_strategy == DenyUncommitted && !found.committed {
            return Err(NotCommitted);
        }
        if found.generation != entity.generation {
            return Err(IsStale);
        }
        Ok(())
    }

    pub(crate) fn new_entity(&mut self) -> InternalEntityKey {
        trace!("creating new entity");
        let index = match self.holes.pop() {
            None => {
                let index = self.allocation_boundary;
                self.allocation_boundary += 1;
                if index >= self.entities.len() {
                    let entity_box_template = EntityBox::new();
                    let new_size = self.entities.len() * 2;
                    let prev = mem::replace(
                        &mut self.entities,
                        vec![entity_box_template; new_size].into_boxed_slice(),
                    );
                    self.entities[0..prev.len()].copy_from_slice(&prev);
                }
                index
            }
            Some(index) => index,
        };

        let entity = self.entities.get_mut(index).unwrap();
        entity.exists = true;
        entity.committed = false;
        entity.generation.increment();

        let key = InternalEntityKey {
            index: EntityIndex {
                index: index as u32,
            },
            generation: entity.generation,
            temp: false,
        };
        trace!("entity created {}", key);
        key
    }

    pub(crate) fn is_not_committed(&self, key: EntityIndex) -> bool {
        self.entities
            .get(key.index as usize)
            .unwrap()
            .committed
            .not()
    }

    pub(crate) fn delete_entity_data(&mut self, key: EntityIndex) {
        trace!("deleting entity data {}", key);
        let key = key.index as usize;
        self.entities.get_mut(key).unwrap().exists = false;
        if key == self.allocation_boundary - 1 {
            self.allocation_boundary -= 1;
        } else {
            self.holes.push(key);
        }
    }

    pub(crate) fn mark_committed(&mut self, entity_key: EntityIndex) {
        trace!("marking entity committed {}", entity_key);
        self.entities
            .get_mut(entity_key.index as usize)
            .unwrap()
            .committed = true;
    }

    pub(crate) fn get_all(&self) -> impl Iterator<Item = InternalEntityKey> + '_ {
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, ent)| ent.exists && ent.committed)
            .map(|(i, ent)| InternalEntityKey {
                index: EntityIndex { index: i as u32 },
                generation: ent.generation,
                temp: false,
            })
    }
}
