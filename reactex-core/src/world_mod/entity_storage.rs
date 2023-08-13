use std::collections::VecDeque;
use std::mem;
use std::ops::Not;
use crate::entity::{EntityGeneration, EntityIndex, InternalEntityKey};
use crate::world_mod::world::EntityError;
use crate::world_mod::world::EntityError::{IsStale, NotCommitted, NotExists};
use crate::world_mod::entity_storage::ValidateUncommitted::DenyUncommitted;

pub(crate) struct EntityStorage {
    entities: Box<[EntityBox]>,
    allocation_boundary: usize,
    holes: VecDeque<usize>,
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
pub enum ValidateUncommitted {
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

    pub fn new_entity(&mut self) -> InternalEntityKey {
        let index = match self.holes.pop_front() {
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

        InternalEntityKey {
            index: EntityIndex {
                index: index as u32,
            },
            generation: entity.generation,
        }
    }

    pub fn is_not_committed(&self, key: EntityIndex) -> bool {
        self.entities
            .get(key.index as usize)
            .unwrap()
            .committed
            .not()
    }

    pub fn delete_entity_data(&mut self, key: EntityIndex) {
        let key = key.index as usize;
        self.entities.get_mut(key).unwrap().exists = false;
        if key == self.allocation_boundary - 1 {
            self.allocation_boundary -= 1;
        } else {
            self.holes.push_back(key);
        }
    }

    pub fn mark_committed(&mut self, entity_key: EntityIndex) {
        self.entities
            .get_mut(entity_key.index as usize)
            .unwrap()
            .committed = true;
    }

    pub fn get_all(&self) -> impl Iterator<Item = InternalEntityKey> + '_ {
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, ent)| ent.exists && ent.committed)
            .map(|(i, ent)| InternalEntityKey {
                index: EntityIndex { index: i as u32 },
                generation: ent.generation,
            })
    }
}
