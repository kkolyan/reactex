use std::collections::HashSet;
use crate::entity::InternalEntityKey;
use crate::world_mod::entity_component_index::EntityComponentIndex;
use crate::filter::events::FilterComponentChange;
use crate::filter::filter_manager::FilterManager;
use crate::opt_tiny_vec::OptTinyVec;
use crate::cause::Cause;

impl FilterManager {
    pub fn on_component_added(
        &mut self,
        entity_component_index: &EntityComponentIndex,
        change: FilterComponentChange,
    ) {
        let filters = self
            .by_component_type
            .get_mut(&change.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());
        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
            let present: HashSet<_> = HashSet::from_iter(
                entity_component_index.get_component_types(change.component_key.entity.index),
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
                matched.insert(change.component_key.entity);
                events = true;
            }
            if let Some(appear_events) = &mut filter.appear_events {
                appear_events
                    .entry(change.component_key.entity)
                    .or_default()
                    // TODO consider get rid of clones
                    .extend(change.causes.iter().cloned());
                events = true;
            }

            if events {
                self.with_new_appear_events.insert(filter.unique_key);
            }
        }
    }

    pub fn on_component_removed(&mut self, change: FilterComponentChange) {
        let filters = self
            .by_component_type
            .get_mut(&change.component_key.component_type)
            .into_iter()
            .flat_map(|it| it.iter());
        for filter in filters {
            let filter = self.owned.get_mut(filter).unwrap();
            let mut events = false;
            if let Some(matched) = &mut filter.matched_entities {
                matched.remove(&change.component_key.entity);
                events = true;
            }
            if let Some(disappear_events) = &mut filter.disappear_events {
                // TODO seems like that's duplicated here and in GenerateDisappearEvents
                disappear_events
                    .entry(change.component_key.entity)
                    .or_default()
                    .extend(change.causes.iter().cloned());
                events = true;
            }
            if events {
                self.with_new_disappear_events.insert(filter.unique_key);
            }
        }
    }

    pub fn on_entity_destroyed(
        &mut self,
        entity: InternalEntityKey,
        causes: OptTinyVec<Cause>,
    ) {
        let relevant_key = self.get_all_entities_filter()
            .map(|filter| {
                if let Some(matched_entities) = &mut filter.matched_entities {
                    matched_entities.remove(&entity);
                }
                if let Some(disappear_events) = &mut filter.disappear_events {
                    disappear_events
                        .entry(entity)
                        .or_default()
                        .extend(causes);
                }
                filter.unique_key
            });
        if let Some(key) = relevant_key {
            self.with_new_disappear_events.insert(key);
        }
    }

    pub fn on_entity_created(
        &mut self,
        entity: InternalEntityKey,
        causes: OptTinyVec<Cause>,
    ) {
        let relevant_key = self.get_all_entities_filter()
            .map(|filter| {
                if let Some(matched_entities) = &mut filter.matched_entities {
                    matched_entities.insert(entity);
                }
                if let Some(appear_events) = &mut filter.appear_events {
                    appear_events
                        .entry(entity)
                        .or_default()
                        .extend(causes);
                }
                filter.unique_key
            });
        if let Some(unique_key) = relevant_key {
            self.with_new_appear_events.insert(unique_key);
        }
    }
}
