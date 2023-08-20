use crate::component::ComponentType;
use crate::filter::events::FilterComponentChange;
use crate::filter::filter::Filter;
use crate::filter::filter_desc::FilterDesc;
use crate::typed_index_vec::TiVec;
use std::collections::HashMap;
use std::collections::HashSet;
use log::trace;
use crate::cause::Cause;
use crate::entity::InternalEntityKey;
use crate::opt_tiny_vec::OptTinyVec;
use crate::world_mod::component_mapping::ComponentMappingStorage;
use crate::world_mod::entity_component_index::EntityComponentIndex;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct InternalFilterKey(pub usize);

#[derive(Default)]
pub(crate) struct FilterManager {
    pub owned: TiVec<InternalFilterKey, Filter>,
    by_key: HashMap<FilterDesc, InternalFilterKey>,
    by_key_ptr: HashMap<*const [ComponentType], InternalFilterKey>,
    pub by_component_type: HashMap<ComponentType, Vec<InternalFilterKey>>,
    all_entities: Option<InternalFilterKey>,
    pub with_new_appear_events: HashSet<InternalFilterKey>,
    pub with_new_disappear_events: HashSet<InternalFilterKey>,
}

// basic operations
impl FilterManager {
    pub fn get_filter_internal(&mut self, key: InternalFilterKey) -> &mut Filter {
        self.owned.get_mut(&key).unwrap()
    }

    pub fn get_all_entities_filter(&mut self) -> Option<&mut Filter> {
        match self.all_entities {
            None => None,
            Some(it) => self.owned.get_mut(&it),
        }
    }

    pub fn get_filter(&mut self, key: FilterDesc) -> &mut Filter {
        let key_ptr = key.component_types as *const _;
        if let Some(filter_index) = self.by_key_ptr.get(&key_ptr) {
            return self.owned.get_mut(filter_index).unwrap();
        }
        if let Some(filter_index) = self.by_key.get_mut(&key).copied() {
            self.by_key_ptr.insert(key_ptr, filter_index);
            return self.owned.get_mut(&filter_index).unwrap();
        }
        let filter_index = self.owned.push_with_key(|index| {
            let index1 = *index;
            Filter {
                criteria: key,
                unique_key: index1,
                matched_entities: None,
                appear_events: None,
                disappear_events: None,
            }
        });
        self.by_key_ptr.insert(key_ptr, filter_index);
        self.by_key.insert(key, filter_index);

        if key.component_types.is_empty() {
            self.all_entities = Some(filter_index);
        } else {
            for component_type in key.component_types {
                self.by_component_type
                    .entry(*component_type)
                    .or_default()
                    .push(filter_index);
            }
        }

        self.owned.get_mut(&filter_index).unwrap()
    }

    pub fn generate_disappear_events(&mut self, component: FilterComponentChange, entity_component_index: &EntityComponentIndex) {
        trace!("generate disappear events {}", component.component_key);
        let filters = self
            .by_component_type
            .get_mut(&component.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());

        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();

            let present: HashSet<_> = HashSet::from_iter(
                entity_component_index.get_component_types(component.component_key.entity.index),
            );
            if !filter.criteria.component_types.iter().all(|it| present.contains(it)) {
                continue;
            }
            if let Some(disappear_events) = &mut filter.disappear_events {
                disappear_events
                    .entry(component.component_key.entity)
                    .or_default()
                    // TODO consider remove clone
                    .extend(component.causes.clone());
                self.with_new_disappear_events.insert(filter.unique_key);
            }
        }
    }

    pub(crate) fn generate_entity_disappear_events(&mut self, entity: InternalEntityKey, causes: OptTinyVec<Cause>) {
        trace!("generate disappear events for entity {}", entity);
        let filters = self
            .all_entities
            .iter();

        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
            if let Some(disappear_events) = &mut filter.disappear_events {
                disappear_events
                    .entry(entity)
                    .or_default()
                    // TODO consider remove clone
                    .extend(causes.clone());
                self.with_new_disappear_events.insert(filter.unique_key);
            }
        }
    }
}
