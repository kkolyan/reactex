use std::borrow::Cow;
use std::mem;

use log::trace;

use crate::{ExecutionResult, World};
use crate::internal::component_key::ComponentKey;
use crate::internal::execution::{invoke_user_code, UserCode};
use crate::internal::filter_manager_events::FilterComponentChange;
use crate::internal::world_extras::ComponentEventType;
use crate::utils::opt_tiny_vec::OptTinyVec;

impl World {

    pub(crate) fn flush_entity_create_actions(&mut self) {
        for (task, causes) in mem::take(&mut self.volatile.entities_to_commit) {
            trace!("flush create entity {}", task);
            self.entity_storage.mark_committed(task.index);
            self.stable.filter_manager.on_entity_created(task, causes);
        }
    }

    pub(crate) fn flush_component_modification(&mut self) {
        for (component_key, modifications) in mem::take(&mut self.volatile.components_to_modify) {
            trace!("flush component notification {}", component_key);
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
            trace!("flushing add component {}", component_key);
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

    pub(crate) fn invoke_disappear_handlers(&mut self, result: &mut ExecutionResult) {
        *result += self.invoke_handlers(ComponentEventType::Disappear);
    }

    pub(crate) fn invoke_appear_handlers(&mut self, result: &mut ExecutionResult) {
        *result += self.invoke_handlers(ComponentEventType::Appear);
    }

    pub(crate) fn flush_entity_destroy_actions(&mut self) {
        for (entity, _) in mem::take(&mut self.volatile.entities_to_destroy.after_disappear) {
            trace!("flush destroy entity {}", entity);
            self.entity_storage.delete_entity_data(entity.index);
            self.stable.filter_manager.on_entity_destroyed(entity);
        }
    }

    pub(crate) fn flush_component_removals(&mut self) {
        for (component_key, causes) in
            mem::take(&mut self.volatile.components_to_delete.after_disappear)
        {
            trace!("flush remove component {}", component_key);
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
            trace!("schedule destroy {}", entity);
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

    pub(crate) fn invoke_signal_handler(&mut self, result: &mut ExecutionResult) {
        if let Some(signal) = self.volatile.signal_queue.signals.pop_front() {
            trace!("triggering signal handlers {}", signal.payload_type_name);
            let manager = self
                .immutable
                .signal_managers
                .get(&signal.payload_type)
                .unwrap();
            *result += manager.invoke(
                signal,
                &mut self.stable,
                &mut self.volatile,
                &mut self.entity_storage,
            );
        }
    }

    fn invoke_handlers(&mut self, event_type: ComponentEventType) -> ExecutionResult {
        let filters = mem::take(match event_type {
            ComponentEventType::Appear => &mut self.stable.filter_manager.with_new_appear_events,
            ComponentEventType::Disappear => {
                &mut self.stable.filter_manager.with_new_disappear_events
            }
        });
        let handlers = match event_type {
            ComponentEventType::Appear => &self.immutable.on_appear,
            ComponentEventType::Disappear => &self.immutable.on_disappear,
        };
        let mut result = ExecutionResult::new();

        for filter in filters {
            if let Some(handlers) = handlers.get(&filter) {
                let filter = &mut self.stable.filter_manager.get_filter_internal(filter);
                let events = match event_type {
                    ComponentEventType::Appear => &mut filter.appear_events,
                    ComponentEventType::Disappear => &mut filter.disappear_events,
                };
                let events = events.as_mut().map(mem::take);
                for handler in handlers {
                    if let Some(events) = &events {
                        for (entity, causes) in events {
                            trace!("triggering event {:?} for {}", event_type, entity);
                            result += invoke_user_code(
                                &mut self.volatile,
                                &self.stable,
                                &mut self.entity_storage,
                                handler.name,
                                causes.iter().cloned(),
                                [UserCode::new(|ctx| {
                                    (handler.callback)(ctx, entity.export())
                                })],
                                |_| {},
                                &(),
                            );
                        }
                    }
                }
            }
        }

        result
    }
}
