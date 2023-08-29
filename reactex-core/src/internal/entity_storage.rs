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
    fn holes_pop(&self, state: &mut TemporaryEntityKeyStorage) -> Option<usize> {
        if self.holes.len() <= state.used_holes {
            return None;
        }
        let result = self
            .holes
            .get(self.holes.len() - state.used_holes - 1)
            .copied();
        state.used_holes += 1;
        return result;
    }

    pub(crate) fn generate_temporary(
        &self,
        state: &mut TemporaryEntityKeyStorage,
    ) -> TempEntityKey {
        let index = match self.holes_pop(state) {
            None => {
                let index = self.allocation_boundary + state.tail_allocations;
                state.tail_allocations += 1;
                index
            }
            Some(index) => index,
        };

        let generation: EntityGeneration = self.entities.get(index)
            .map(|it| it.generation)
            .unwrap_or_else(EntityGeneration::new);

        let key = InternalEntityKey {
            index: EntityIndex {
                index: index as u32,
            },
            generation: generation.to_next_generation(),
            temp: false,
        };
        TempEntityKey { inner: key }
    }

    pub(crate) fn persist_generated(&mut self, input: TempEntityKey) -> InternalEntityKey {
        trace!("persisting entity {}", input.inner);

        let index = input.inner.index;

        let generation = self
            .entities
            .get_mut(input.inner.index.index as usize)
            .map(|it| it.generation)
            .unwrap_or(EntityGeneration::new());
        assert_eq!(generation.to_next_generation(), input.inner.generation);

        if self.holes.last().copied() == Some(index.index as usize) {
            self.holes.pop();
        } else {
            if self.entities.len() == index.index as usize {
                self.extend();
            }
            assert!(self.entities.len() > index.index as usize);
            self.allocation_boundary += 1;
        }

        let entity = self
            .entities
            .get_mut(input.inner.index.index as usize)
            .unwrap();
        entity.exists = true;
        entity.committed = false;
        entity.generation = input.inner.generation;

        input.inner
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
        let mut state = TemporaryEntityKeyStorage::new();
        let key = self.generate_temporary(&mut state);
        let key = self.persist_generated(key);
        trace!("entity created {}", key);
        key
    }

    fn extend(&mut self) {
        let new_size = self.entities.len() * 2;
        let entity_box_template = EntityBox::new();
        let prev = mem::replace(
            &mut self.entities,
            vec![entity_box_template; new_size].into_boxed_slice(),
        );
        self.entities[0..prev.len()].copy_from_slice(&prev);
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
