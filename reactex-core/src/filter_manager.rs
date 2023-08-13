use std::collections::{HashMap, HashSet};
use crate::entity::{InternalEntityKey};
use crate::{ComponentType, FilterKey};
use crate::opt_tiny_vec::OptTinyVec;
use crate::typed_index_vec::{TiVec, TiVecKey};
use crate::world::{Cause, ComponentKey, World};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct FilterIndex(pub usize);

impl TiVecKey for FilterIndex {
    fn from_index(index: usize) -> Self { FilterIndex(index) }
    fn as_index(&self) -> usize { self.0 }
}

pub struct FilterManager {
    owned: TiVec<FilterIndex, Filter>,
    by_key: HashMap<FilterKey, FilterIndex>,
    by_key_ptr: HashMap<*const [ComponentType], FilterIndex>,
    by_component_type: HashMap<ComponentType, Vec<FilterIndex>>,
    all_entities: Option<FilterIndex>,
    with_new_appear_events: HashSet<FilterIndex>,
    with_new_disappear_events: HashSet<FilterIndex>,
}

pub(crate) struct Filter {
    pub key: FilterKey,
    index: FilterIndex,
    pub(crate) matched_entities: Option<HashSet<InternalEntityKey>>,
    pub(crate) appear_events: Option<HashMap<InternalEntityKey, OptTinyVec<Cause>>>,
    pub(crate) disappear_events: Option<HashMap<InternalEntityKey, OptTinyVec<Cause>>>,
    fill: fn(&World, &mut Filter),
}

impl Filter {
    fn new(key: FilterKey, index: FilterIndex, fill: fn(&World, &mut Filter)) -> Filter {
        Filter {
            key,
            index,
            matched_entities: None,
            appear_events: None,
            disappear_events: None,
            fill,
        }
    }
}

impl Filter {
    pub(crate) fn track_matched_entities(&mut self) -> &mut HashSet<InternalEntityKey> {
        self.matched_entities.get_or_insert_with(|| Default::default())
    }
}

impl FilterManager {
    pub(crate) fn get_filter(&mut self, key: FilterKey) -> &mut Filter {
        let key_ptr = key.component_types as *const _;
        if let Some(filter_index) = self.by_key_ptr.get(&key_ptr) {
            return self.owned.get_mut(filter_index).unwrap();
        }
        if let Some(filter_index) = self.by_key.get_mut(&key).copied() {
            self.by_key_ptr.insert(key_ptr, filter_index);
            return self.owned.get_mut(&filter_index).unwrap();
        }
        let filter_index = self.owned.push_with_key(|index| Filter::new(key, *index, pre_fill_matched_entities));
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

fn pre_fill_matched_entities(world: &World, filter: &mut Filter) {
    for entity in world.get_all_entities() {
        let matches = filter.key.component_types.iter()
            .all(|component_type| world.has_component_no_validation(entity.index, *component_type));
        if matches {
            filter.matched_entities.as_mut().unwrap().insert(entity);
        }
    }
}

impl FilterManager {
    pub(crate) fn generate_disappear_events(&mut self, component: ComponentChangeKey) {
        let filters = self.by_component_type
            .get_mut(&component.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());

        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
            if let Some(disappear_events) = &mut filter.disappear_events {
                disappear_events.entry(component.component_key.entity)
                    .or_default()
                    // TODO consider remove clone
                    .extend(component.causes.clone());
                self.with_new_disappear_events.insert(filter.index);
            }
        }
    }
}


impl FilterManager {
    pub(crate) fn on_component_added<F, I>(&mut self, get_component_types: F, component: ComponentAddKey)
        where F: Fn(InternalEntityKey) -> I,
              I: Iterator<Item=ComponentType>,
    {
        let filters = self.by_component_type
            .get_mut(&component.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());
        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
            let present: HashSet<_> = HashSet::from_iter(get_component_types(component.component_key.entity));
            let matches = filter.key.component_types.iter().all(|ct| present.contains(ct));

            if !matches {
                continue;
            }
            let mut events = false;

            if let Some(matched) = &mut filter.matched_entities {
                matched.insert(component.component_key.entity);
                events = true;
            }
            if let Some(appear_events) = &mut filter.appear_events {
                appear_events.entry(component.component_key.entity)
                    .or_default()
                    // TODO consider get rid of clones
                    .extend(component.causes.iter().cloned());
                events = true;
            }

            if events {
                self.with_new_appear_events.insert(filter.index);
            }
        }
    }

    pub(crate) fn on_component_removed(&mut self, component: ComponentRemoveKey) {
        let filters = self.by_component_type
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
                disappear_events.entry(component.component_key.entity)
                    .or_default()
                    .extend(component.causes.iter().cloned());
                events = true;
            }
            if events {
                self.with_new_disappear_events.insert(filter.index);
            }
        }
    }
}

impl FilterManager {
    pub(crate) fn on_entity_destroyed(&mut self, entity: InternalEntityKey, causes: OptTinyVec<Cause>) {
        if let Some(filter) = self.all_entities {
            let filter = self.owned.get_mut(&filter).unwrap();
            if let Some(matched_entities) = &mut filter.matched_entities {
                matched_entities.remove(&entity);
            }
            if let Some(disappear_events) = &mut filter.disappear_events {
                disappear_events.entry(entity)
                    .or_default()
                    .extend(causes.into_iter());
            }
            self.with_new_disappear_events.insert(filter.index);
        }
    }

    pub(crate) fn on_entity_created(&mut self, entity: InternalEntityKey, causes: OptTinyVec<Cause>) {
        if let Some(filter) = self.all_entities {
            let filter = self.owned.get_mut(&filter).unwrap();
            if let Some(matched_entities) = &mut filter.matched_entities {
                matched_entities.insert(entity);
            }
            if let Some(appear_events) = &mut filter.appear_events {
                appear_events.entry(entity)
                    .or_default()
                    .extend(causes.into_iter());
            }
            self.with_new_appear_events.insert(filter.index);
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
