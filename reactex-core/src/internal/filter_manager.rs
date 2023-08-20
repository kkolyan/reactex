use crate::component::ComponentType;
use crate::filter::FilterDesc;
use crate::internal::cause::Cause;
use crate::internal::entity_component_index::EntityComponentIndex;
use crate::internal::filter::Filter;
use crate::internal::filter_manager_events::FilterComponentChange;
use crate::internal::world_extras::InternalEntityKey;
use crate::utils::opt_tiny_vec::OptTinyVec;
use crate::utils::typed_index_vec::TiVec;
use crate::utils::typed_index_vec::TiVecKey;
use log::trace;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct InternalFilterKey(pub(crate) usize);

#[derive(Default)]
pub(crate) struct FilterManager {
    pub(crate) owned: TiVec<InternalFilterKey, Filter>,
    by_key: HashMap<FilterDesc, InternalFilterKey>,
    by_key_ptr: HashMap<*const [ComponentType], InternalFilterKey>,
    pub(crate) by_component_type: HashMap<ComponentType, Vec<InternalFilterKey>>,
    all_entities: Option<InternalFilterKey>,
    pub(crate) with_new_appear_events: HashSet<InternalFilterKey>,
    pub(crate) with_new_disappear_events: HashSet<InternalFilterKey>,
}

impl TiVecKey for InternalFilterKey {
    fn from_index(index: usize) -> Self {
        InternalFilterKey(index)
    }
    fn as_index(&self) -> usize {
        self.0
    }
}

// basic operations
impl FilterManager {
    pub(crate) fn get_filter_internal(&mut self, key: InternalFilterKey) -> &mut Filter {
        self.owned.get_mut(&key).unwrap()
    }

    pub(crate) fn get_all_entities_filter(&mut self) -> Option<&mut Filter> {
        match self.all_entities {
            None => None,
            Some(it) => self.owned.get_mut(&it),
        }
    }

    pub(crate) fn get_filter(&self, key: FilterDesc) -> &Filter {
        let key_ptr = key.component_types as *const _;
        if let Some(filter_index) = self.by_key_ptr.get(&key_ptr) {
            return self.owned.get(filter_index).unwrap();
        }
        if let Some(filter_index) = self.by_key.get(&key).copied() {
            return self.owned.get(&filter_index).unwrap();
        }
        panic!("filter not initialized: {}", key)
    }

    pub(crate) fn get_filter_by_key(&self, key: InternalFilterKey) -> &Filter {
        return self.owned.get(&key).unwrap();
    }

    pub(crate) fn get_filter_mut(&mut self, key: FilterDesc) -> &mut Filter {
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

    pub(crate) fn generate_disappear_events(
        &mut self,
        component: FilterComponentChange,
        entity_component_index: &EntityComponentIndex,
    ) {
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
            if !filter
                .criteria
                .component_types
                .iter()
                .all(|it| present.contains(it))
            {
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

    pub(crate) fn generate_entity_disappear_events(
        &mut self,
        entity: InternalEntityKey,
        causes: OptTinyVec<Cause>,
    ) {
        trace!("generate disappear events for entity {}", entity);
        let filters = self.all_entities.iter();

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
