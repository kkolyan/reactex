use crate::component::ComponentType;
use crate::filter::FilterDesc;
use crate::internal::cause::Cause;
use crate::internal::component_key::ComponentKey;
use crate::internal::filter_manager_events::FilterComponentChange;
use crate::internal::world_extras::ComponentEventType;
use crate::internal::world_pipeline;
use crate::internal::world_stable::StableWorld;
use crate::internal::world_volatile::VolatileWorld;
use crate::utils::opt_tiny_vec::OptTinyVec;
use crate::Ctx;
use log::trace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;
use std::sync::Mutex;
use std::sync::RwLock;

pub(crate) static COMPONENT_TYPE_REGISTRATIONS: Mutex<Vec<fn(&mut World)>> = Mutex::new(Vec::new());

pub(crate) static COMPONENT_NAMES: RwLock<Option<HashMap<ComponentType, &'static str>>> =
    RwLock::new(None);

pub(crate) static QUERIES: Mutex<Option<HashSet<FilterDesc>>> = Mutex::new(None);

pub struct World {
    pub(crate) volatile: VolatileWorld,
    pub(crate) stable: StableWorld,
}

impl World {
    pub(crate) fn new() -> Self {
        let mut world = Self {
            volatile: VolatileWorld::new(),
            stable: StableWorld::new(),
        };
        for registration in COMPONENT_TYPE_REGISTRATIONS.lock().unwrap().iter() {
            registration(&mut world);
        }

        for filter in QUERIES.lock().unwrap().iter().flatten() {
            world.register_filter(*filter);
        }

        world_pipeline::configure_pipeline(&mut world);
        world
    }

    fn register_filter(&mut self, filter: FilterDesc) {
        self.stable
            .filter_manager
            .get_filter(filter)
            .track_matched_entities(
                self.stable.entity_storage.get_mut(),
                &self.stable.component_mappings,
            );
    }

    pub(crate) fn flush_entity_create_actions(&mut self) {
        for (task, causes) in mem::take(&mut self.volatile.entities_to_commit) {
            self.stable
                .entity_storage
                .get_mut()
                .mark_committed(task.index);
            self.stable.filter_manager.on_entity_created(task, causes);
        }
    }

    pub(crate) fn flush_component_modification(&mut self) {
        for (component_key, modifications) in mem::take(&mut self.volatile.components_to_modify) {
            let data = self
                .stable
                .get_component_mapping_mut(component_key.component_type)
                .get(&component_key.entity.index);
            let Some(&data) = data else {
                continue;
            };
            let value = self
                .stable
                .component_data
                .get_pool_mut(component_key.component_type)
                .get_any_mut(&data);
            let Some(value) = value else {
                continue;
            };
            for modification in modifications {
                (modification.callback)(value);
            }
        }
    }

    pub(crate) fn flush_component_addition(&mut self) {
        for (component_key, versions) in mem::take(&mut self.volatile.components_to_add) {
            let mut versions = versions.into_iter();

            let chosen_version = versions.next().unwrap();

            let mut all_causes = OptTinyVec::single(chosen_version.cause);
            all_causes.extend(versions.map(|it| it.cause));

            let chosen_version = self
                .stable
                .component_data_pumps
                .get(&component_key.component_type)
                .unwrap()
                .do_move(
                    self.volatile
                        .component_data_uncommitted
                        .get_pool_mut(component_key.component_type),
                    self.stable
                        .component_data
                        .get_pool_mut(component_key.component_type),
                    &chosen_version.data,
                );

            let mapping = self
                .stable
                .get_component_mapping_mut(component_key.component_type)
                .entry(component_key.entity.index)
                .or_insert(chosen_version);
            assert_eq!(
                *mapping, chosen_version,
                "attempt to mark committed as committed"
            );

            self.volatile
                .entity_component_index
                .add_component_type(component_key.entity.index, component_key.component_type);

            self.stable.filter_manager.on_component_added(
                &self.volatile.entity_component_index,
                FilterComponentChange {
                    component_key,
                    causes: all_causes,
                },
            );
        }
        // deleting cancelled components (which entity or themselves was deleted at the same transaction)
        self.volatile.component_data_uncommitted.clear();
    }

    pub(crate) fn invoke_disappear_handlers(&mut self) {
        Self::invoke_handlers(
            ComponentEventType::Disappear,
            &mut self.stable,
            &mut self.volatile,
        );
    }

    pub(crate) fn invoke_appear_handlers(&mut self) {
        Self::invoke_handlers(
            ComponentEventType::Appear,
            &mut self.stable,
            &mut self.volatile,
        );
    }

    fn invoke_handlers(
        event_type: ComponentEventType,
        stable: &mut StableWorld,
        volatile: &mut VolatileWorld,
    ) {
        let filters = mem::take(match event_type {
            ComponentEventType::Appear => &mut stable.filter_manager.with_new_appear_events,
            ComponentEventType::Disappear => &mut stable.filter_manager.with_new_disappear_events,
        });
        let handlers = match event_type {
            ComponentEventType::Appear => &stable.on_appear,
            ComponentEventType::Disappear => &stable.on_disappear,
        };
        for filter in filters {
            if let Some(handlers) = handlers.get(&filter) {
                for handler in handlers {
                    let filter = stable.filter_manager.get_filter_internal(filter);
                    let events = match event_type {
                        ComponentEventType::Appear => &mut filter.appear_events,
                        ComponentEventType::Disappear => &mut filter.disappear_events,
                    };
                    let events = events.as_mut().map(mem::take);
                    if let Some(events) = events {
                        for (entity, causes) in events {
                            trace!("invoke event {:?} for {}", event_type, entity);
                            let new_cause = Cause::consequence(handler.name, causes);
                            let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);
                            let ctx = Ctx {
                                signal: &(),
                                stable,
                                volatile: &RefCell::new(volatile),
                            };
                            (handler.callback)(ctx, entity.export());
                            volatile.current_cause = prev_cause;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn flush_entity_destroy_actions(&mut self) {
        for (entity, causes) in mem::take(&mut self.volatile.entities_to_destroy.after_disappear) {
            self.stable
                .entity_storage
                .get_mut()
                .delete_entity_data(entity.index);
            self.stable
                .filter_manager
                .on_entity_destroyed(entity, causes);
        }
    }

    pub(crate) fn flush_component_removals(&mut self) {
        for (component_key, causes) in
            mem::take(&mut self.volatile.components_to_delete.after_disappear)
        {
            let data_key = self
                .stable
                .component_mappings
                .data_by_entity_by_type
                .get_mut(&component_key.component_type)
                .unwrap()
                .remove(&component_key.entity.index);
            if let Some(data_key) = data_key {
                self.stable
                    .component_data
                    .get_pool_mut(component_key.component_type)
                    .del(&data_key);
            }

            self.volatile
                .entity_component_index
                .delete_component_type(component_key.entity.index, component_key.component_type);
            self.stable
                .filter_manager
                .on_component_removed(FilterComponentChange {
                    component_key,
                    causes,
                })
        }
    }

    pub(crate) fn generate_disappear_events(&mut self) {
        for (component_key, causes) in
            mem::take(&mut self.volatile.components_to_delete.before_disappear)
        {
            self.stable.filter_manager.generate_disappear_events(
                FilterComponentChange {
                    component_key,
                    // TODO consider avoid cloning
                    causes: causes.clone(),
                },
                &self.volatile.entity_component_index,
            );
            self.volatile
                .components_to_delete
                .after_disappear
                .insert(component_key, causes);
        }
    }

    pub(crate) fn schedule_destroyed_entities_component_removal(&mut self) {
        for (entity, causes) in mem::take(&mut self.volatile.entities_to_destroy.before_disappear) {
            for component_type in self
                .volatile
                .entity_component_index
                .get_component_types(entity.index)
            {
                let component_key = ComponentKey {
                    entity,
                    component_type,
                };
                let existing_causes = self
                    .volatile
                    .components_to_delete
                    .before_disappear
                    .entry(component_key)
                    .or_default();
                for cause in causes.clone() {
                    trace!(
                        "scheduling removal of {} due to entity destroy",
                        component_key
                    );
                    existing_causes.push(cause);
                }
            }
            self.volatile
                .entities_to_destroy
                .after_disappear
                .entry(entity)
                .or_default()
                .extend(causes.clone());

            self.stable
                .filter_manager
                .generate_entity_disappear_events(entity, causes);
        }
    }

    pub(crate) fn invoke_signal_handler(&mut self) {
        if let Some(signal) = self.volatile.signal_queue.signals.pop_front() {
            let manager = self
                .stable
                .signal_managers
                .get(&signal.payload_type)
                .unwrap();
            manager.invoke(signal, &self.stable, &mut self.volatile);
        }
    }
}
