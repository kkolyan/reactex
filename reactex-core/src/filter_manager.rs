use crate::entity::InternalEntityKey;
use crate::entity_component_index::EntityComponentIndex;
use crate::entity_storage::EntityStorage;
use crate::opt_tiny_vec::OptTinyVec;
use crate::typed_index_vec::TiVec;
use crate::typed_index_vec::TiVecKey;
use crate::world::Cause;
use crate::world::ComponentKey;
use crate::world::ComponentMappings;
use crate::ComponentType;
use crate::FilterKey;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct InternalFilterKey(pub usize);

impl TiVecKey for InternalFilterKey {
    fn from_index(index: usize) -> Self {
        InternalFilterKey(index)
    }
    fn as_index(&self) -> usize {
        self.0
    }
}

pub(crate) struct FilterIter<'a, TKeys> {
    source: &'a mut TiVec<InternalFilterKey, Filter>,
    keys: TKeys,
}

impl<'a, I> FilterIter<'a, I>
where
    I: Iterator<Item = InternalFilterKey>,
{
    pub fn next(&mut self) -> Option<&mut Filter> {
        match self.keys.next() {
            None => None,
            Some(it) => self.source.get_mut(&it),
        }
    }
}

pub struct FilterManager {
    owned: TiVec<InternalFilterKey, Filter>,
    by_key: HashMap<FilterKey, InternalFilterKey>,
    by_key_ptr: HashMap<*const [ComponentType], InternalFilterKey>,
    by_component_type: HashMap<ComponentType, Vec<InternalFilterKey>>,
    all_entities: Option<InternalFilterKey>,
    with_new_appear_events: HashSet<InternalFilterKey>,
    with_new_disappear_events: HashSet<InternalFilterKey>,
}

pub(crate) struct Filter {
    pub criteria: FilterKey,
    pub unique_key: InternalFilterKey,
    pub(crate) matched_entities: Option<HashSet<InternalEntityKey>>,
    pub(crate) appear_events: Option<HashMap<InternalEntityKey, OptTinyVec<Cause>>>,
    pub(crate) disappear_events: Option<HashMap<InternalEntityKey, OptTinyVec<Cause>>>,
}

impl Filter {
}

impl Filter {}

impl Filter {
    pub(crate) fn track_matched_entities(
        &mut self,
        entity_storage: &EntityStorage,
        component_mappings: &ComponentMappings,
    ) -> &mut HashSet<InternalEntityKey> {
        if self.matched_entities.is_none() {
            self.matched_entities = Some(Default::default());
            pre_fill_matched_entities(self, entity_storage, component_mappings);
        }
        self.matched_entities.as_mut().unwrap()
    }

    pub(crate) fn track_appear_events(&mut self) {
        if self.appear_events.is_none() {
            self.appear_events = Some(Default::default());
        }
    }

    pub(crate) fn track_disappear_events(&mut self) {
        if self.disappear_events.is_none() {
            self.disappear_events = Some(Default::default());
        }
    }
}

impl FilterManager {
    pub(crate) fn take_with_new_disappear_events(
        &mut self,
    ) -> FilterIter<impl Iterator<Item = InternalFilterKey> + '_> {
        FilterIter {
            source: &mut self.owned,
            keys: mem::take(&mut self.with_new_disappear_events).into_iter(),
        }
    }

    pub(crate) fn take_with_new_appear_events(
        &mut self,
    ) -> FilterIter<impl Iterator<Item = InternalFilterKey> + '_> {
        FilterIter {
            source: &mut self.owned,
            keys: mem::take(&mut self.with_new_appear_events).into_iter(),
        }
    }

    pub(crate) fn get_filter_internal(&mut self, key: InternalFilterKey) -> &mut Filter {
        self.owned.get_mut(&key).unwrap()
    }

    pub(crate) fn get_filter(&mut self, key: FilterKey) -> &mut Filter {
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
}

fn pre_fill_matched_entities(
    filter: &mut Filter,
    entity_storage: &EntityStorage,
    component_mappings: &ComponentMappings,
) {
    for entity in entity_storage.get_all() {
        let matches = filter
            .criteria
            .component_types
            .iter()
            .all(|component_type| {
                component_mappings.has_component_no_validation(entity.index, *component_type)
            });
        if matches {
            filter.matched_entities.as_mut().unwrap().insert(entity);
        }
    }
}

impl FilterManager {
    pub(crate) fn generate_disappear_events(&mut self, component: ComponentChangeKey) {
        let filters = self
            .by_component_type
            .get_mut(&component.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());

        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
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
}

impl FilterManager {
    pub(crate) fn on_component_added(
        &mut self,
        entity_component_index: &EntityComponentIndex,
        component: ComponentAddKey,
    ) {
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
            let matches = filter
                .criteria
                .component_types
                .iter()
                .all(|ct| present.contains(ct));

            if !matches {
                continue;
            }
            let mut events = false;

            if let Some(matched) = &mut filter.matched_entities {
                matched.insert(component.component_key.entity);
                events = true;
            }
            if let Some(appear_events) = &mut filter.appear_events {
                appear_events
                    .entry(component.component_key.entity)
                    .or_default()
                    // TODO consider get rid of clones
                    .extend(component.causes.iter().cloned());
                events = true;
            }

            if events {
                self.with_new_appear_events.insert(filter.unique_key);
            }
        }
    }

    pub(crate) fn on_component_removed(&mut self, component: ComponentRemoveKey) {
        let filters = self
            .by_component_type
            .get_mut(&component.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());
        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
            let mut events = false;
            if let Some(matched) = &mut filter.matched_entities {
                matched.remove(&component.component_key.entity);
                events = true;
            }
            if let Some(disappear_events) = &mut filter.disappear_events {
                // TODO seems like that's duplicated here and in GenerateDisappearEvents
                disappear_events
                    .entry(component.component_key.entity)
                    .or_default()
                    .extend(component.causes.iter().cloned());
                events = true;
            }
            if events {
                self.with_new_disappear_events.insert(filter.unique_key);
            }
        }
    }
}

impl FilterManager {
    pub(crate) fn on_entity_destroyed(
        &mut self,
        entity: InternalEntityKey,
        causes: OptTinyVec<Cause>,
    ) {
        if let Some(filter) = self.all_entities {
            let filter = self.owned.get_mut(&filter).unwrap();
            if let Some(matched_entities) = &mut filter.matched_entities {
                matched_entities.remove(&entity);
            }
            if let Some(disappear_events) = &mut filter.disappear_events {
                disappear_events
                    .entry(entity)
                    .or_default()
                    .extend(causes.into_iter());
            }
            self.with_new_disappear_events.insert(filter.unique_key);
        }
    }

    pub(crate) fn on_entity_created(
        &mut self,
        entity: InternalEntityKey,
        causes: OptTinyVec<Cause>,
    ) {
        if let Some(filter) = self.all_entities {
            let filter = self.owned.get_mut(&filter).unwrap();
            if let Some(matched_entities) = &mut filter.matched_entities {
                matched_entities.insert(entity);
            }
            if let Some(appear_events) = &mut filter.appear_events {
                appear_events
                    .entry(entity)
                    .or_default()
                    .extend(causes.into_iter());
            }
            self.with_new_appear_events.insert(filter.unique_key);
        }
    }
}

pub(crate) struct ComponentChangeKey {
    pub component_key: ComponentKey,
    pub causes: OptTinyVec<Cause>,
}

pub(crate) struct ComponentAddKey {
    pub component_key: ComponentKey,
    pub(crate) causes: OptTinyVec<Cause>,
}

pub(crate) struct ComponentRemoveKey {
    pub(crate) component_key: ComponentKey,
    pub(crate) causes: OptTinyVec<Cause>,
}

impl FilterManager {
    pub(crate) fn new() -> FilterManager {
        FilterManager {
            owned: TiVec::new(),
            by_key: Default::default(),
            by_key_ptr: Default::default(),
            by_component_type: Default::default(),
            all_entities: None,
            with_new_appear_events: Default::default(),
            with_new_disappear_events: Default::default(),
        }
    }
}
