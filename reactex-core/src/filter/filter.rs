use std::collections::{HashMap, HashSet};
use crate::entity::InternalEntityKey;
use crate::world_mod::entity_storage::EntityStorage;
use crate::filter::filter_desc::FilterDesc;
use crate::filter::filter_manager::InternalFilterKey;
use crate::opt_tiny_vec::OptTinyVec;
use crate::cause::Cause;
use crate::world_mod::component_mapping::ComponentMappingStorage;

pub(crate) struct Filter {
    pub criteria: FilterDesc,
    pub unique_key: InternalFilterKey,
    pub(crate) matched_entities: Option<HashSet<InternalEntityKey>>,
    pub(crate) appear_events: Option<HashMap<InternalEntityKey, OptTinyVec<Cause>>>,
    pub(crate) disappear_events: Option<HashMap<InternalEntityKey, OptTinyVec<Cause>>>,
}

impl Filter {
    pub fn track_matched_entities(
        &mut self,
        entity_storage: &EntityStorage,
        component_mappings: &ComponentMappingStorage,
    ) -> &mut HashSet<InternalEntityKey> {
        if self.matched_entities.is_none() {
            self.matched_entities = Some(Default::default());
            self.pre_fill_matched_entities(entity_storage, component_mappings);
        }
        self.matched_entities.as_mut().unwrap()
    }

    fn pre_fill_matched_entities(
        &mut self,
        entity_storage: &EntityStorage,
        component_mappings: &ComponentMappingStorage,
    ) {
        for entity in entity_storage.get_all() {
            let matches = self
                .criteria
                .component_types
                .iter()
                .all(|component_type| {
                    component_mappings.has_component_no_validation(entity.index, *component_type)
                });
            if matches {
                self.matched_entities.as_mut().unwrap().insert(entity);
            }
        }
    }

    pub fn track_appear_events(&mut self) {
        if self.appear_events.is_none() {
            self.appear_events = Some(Default::default());
        }
    }

    pub fn track_disappear_events(&mut self) {
        if self.disappear_events.is_none() {
            self.disappear_events = Some(Default::default());
        }
    }
}
