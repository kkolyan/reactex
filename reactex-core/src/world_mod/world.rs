use crate::cause::Cause;
use crate::component::ComponentType;
use crate::component::StaticComponentType;
use crate::entity::EntityIndex;
use crate::entity::EntityKey;
use crate::entity::InternalEntityKey;
use crate::filter::events::FilterComponentChange;
use crate::filter::filter_desc::FilterDesc;
use crate::filter::filter_manager::FilterManager;
use crate::filter::filter_manager::InternalFilterKey;
use crate::filter::filter_manager_iter::FilterIter;
use crate::opt_tiny_vec::OptTinyVec;
use crate::pools::AbstractPool;
use crate::world_mod::component_pool_manager::{ComponentDataKey, TempComponentDataKey};
use crate::world_mod::component_mapping::ComponentMappingStorage;
use crate::world_mod::entity_component_index::EntityComponentIndex;
use crate::world_mod::entity_storage::EntityStorage;
use crate::world_mod::entity_storage::ValidateUncommitted::AllowUncommitted;
use crate::world_mod::entity_storage::ValidateUncommitted::DenyUncommitted;
use crate::world_mod::execution::Step;
use crate::world_mod::pipeline;
use crate::world_mod::signal_manager::AbstractSignalManager;
use crate::world_mod::signal_manager::SignalManager;
use crate::world_mod::signal_queue::SignalQueue;
use crate::world_mod::signal_sender::SignalSender;
use crate::world_mod::signal_storage::SignalDataKey;
use crate::world_mod::signal_storage::SignalStorage;
use crate::world_mod::world::ComponentNotFoundError::Data;
use crate::world_mod::world::ComponentNotFoundError::Mapping;
use justerror::Error;
use std::any::{TypeId};
use std::collections::HashMap;
use std::mem;
use crate::pool_pump::{AbstractPoolPump, SpecificPoolPump};
use crate::world_mod::component_pool_manager::ComponentPoolManager;

pub struct World {
    pub(crate) entity_storage: EntityStorage,
    entity_component_index: EntityComponentIndex,
    pub(crate) entities_to_destroy: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    entities_to_commit: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    pub(crate) components_to_delete: DeleteQueue,
    pub(crate) components_to_add: HashMap<ComponentKey, OptTinyVec<ComponentAdd>>,
    pub(crate) component_data: ComponentPoolManager<ComponentDataKey>,
    pub(crate) component_data_uncommitted: ComponentPoolManager<TempComponentDataKey>,
    pub(crate) component_mappings: ComponentMappingStorage,
    current_cause: Cause,
    pub(crate) filter_manager: FilterManager,
    pub(crate) sequence: Vec<Step>,
    pub(crate) on_appear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) on_disappear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) signal_queue: SignalQueue,
    signal_storage: SignalStorage,
    signal_managers: HashMap<TypeId, Box<dyn AbstractSignalManager>>,
    component_data_pumps: HashMap<ComponentType, Box<dyn AbstractPoolPump<TempComponentDataKey, ComponentDataKey>>>,
}

impl World {
    pub fn signal<T: 'static>(&mut self, payload: T) {
        let mut sender = SignalSender {
            signal_queue: &mut self.signal_queue,
            current_cause: &self.current_cause,
            signal_storage: &mut self.signal_storage,
        };
        sender.signal(payload);
    }
}

impl World {
    pub(crate) fn get_signal_manager<T: 'static>(&mut self) -> &mut SignalManager<T> {
        self.signal_managers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<SignalManager<T>>::default())
            .as_any_mut()
            .try_specialize::<T>()
            .unwrap()
    }
}

impl World {
    pub fn query(&mut self, filter: FilterDesc, mut callback: impl FnMut(EntityKey)) {
        for matched_entity in self
            .filter_manager
            .get_filter(filter)
            .track_matched_entities(&self.entity_storage, &self.component_mappings)
            .iter()
        {
            callback(matched_entity.export());
        }
    }
}

#[allow(clippy::new_without_default)]
impl World {
    pub fn new() -> Self {
        let mut world = World {
            entity_storage: EntityStorage::with_capacity(512),
            entity_component_index: EntityComponentIndex::new(512, 8),
            entities_to_destroy: Default::default(),
            entities_to_commit: Default::default(),
            components_to_delete: Default::default(),
            components_to_add: Default::default(),
            component_data: Default::default(),
            component_data_uncommitted: Default::default(),
            component_mappings: Default::default(),
            current_cause: Cause::initial(),
            filter_manager: Default::default(),
            sequence: vec![],
            on_appear: Default::default(),
            on_disappear: Default::default(),
            signal_queue: Default::default(),
            signal_storage: SignalStorage::new(),
            signal_managers: Default::default(),
            component_data_pumps: Default::default(),
        };
        pipeline::configure_pipeline(&mut world);
        world
    }
}

#[derive(Default)]
pub struct DeleteQueue {
    pub(crate) before_disappear: HashMap<ComponentKey, OptTinyVec<Cause>>,
    after_disappear: HashMap<ComponentKey, OptTinyVec<Cause>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentKey {
    pub(crate) entity: InternalEntityKey,
    pub(crate) component_type: ComponentType,
}

impl ComponentKey {
    pub(crate) fn of<T: StaticComponentType>(entity: InternalEntityKey) -> ComponentKey {
        ComponentKey {
            entity,
            component_type: T::get_component_type(),
        }
    }
}

pub struct ComponentAdd {
    data: TempComponentDataKey,
    cause: Cause,
}

pub struct EventHandler {
    pub(crate) name: &'static str,
    pub(crate) callback: Box<dyn Fn(EntityKey)>,
}

pub struct Signal {
    pub payload_type: TypeId,
    pub data_key: SignalDataKey,
    pub cause: Cause,
}

#[Error]
#[derive(Eq, PartialEq)]
pub enum WorldError {
    Entity(#[from] EntityError),
    Component(#[from] ComponentError),
}

pub type WorldResult<T = ()> = Result<T, WorldError>;

#[Error]
#[derive(Eq, PartialEq)]
pub enum ComponentError {
    NotFound(#[from] ComponentNotFoundError),
}

#[Error]
#[derive(Eq, PartialEq)]
pub enum ComponentNotFoundError {
    Mapping,
    Data,
}

impl World {
    pub fn modify_component<T: StaticComponentType>(
        &mut self,
        entity: EntityKey,
        change: impl FnOnce(&mut T),
    ) -> WorldResult {
        let entity = entity
            .validate(&self.entity_storage, DenyUncommitted)?
            .index;
        let data = *self
            .get_component_mapping_mut(T::get_component_type())
            .get(&entity)
            .ok_or(ComponentError::NotFound(Mapping))?;
        let value = self
            .component_data
            .get_pool_or_create()
            .get_mut(&data)
            .ok_or(ComponentError::NotFound(Data))?;
        change(value);
        Ok(())
    }

    pub fn add_component<T: StaticComponentType>(
        &mut self,
        entity: EntityKey,
        component: T,
    ) -> WorldResult {
        let entity = entity.validate(&self.entity_storage, AllowUncommitted)?;

        self.component_data_pumps
            .entry(T::get_component_type())
            .or_insert(Box::<SpecificPoolPump<TempComponentDataKey, ComponentDataKey, T>>::default());

        self.component_data.get_pool_or_create::<T>();

        let data = self.component_data_uncommitted
            .get_pool_or_create::<T>()
            .add(component);

        self.components_to_add
            .entry(ComponentKey::of::<T>(entity))
            .or_default()
            .push(ComponentAdd {
                data,
                cause: self.current_cause.clone(),
            });

        Ok(())
    }

    pub fn remove_component<T: StaticComponentType>(&mut self, entity: EntityKey) -> WorldResult {
        let entity = entity.validate(&self.entity_storage, DenyUncommitted)?;

        let removed_uncommitted = self
            .components_to_add
            .remove(&ComponentKey::of::<T>(entity))
            .is_some();
        if !removed_uncommitted {
            self.components_to_delete
                .before_disappear
                .entry(ComponentKey::of::<T>(entity))
                .or_default()
                .push(self.current_cause.clone());
        }
        Ok(())
    }
}

impl World {
    pub fn create_entity(&mut self) -> EntityKey {
        let entity = self.entity_storage.new_entity();
        self.entity_component_index.add_entity(entity.index);
        self.entities_to_commit
            .entry(entity)
            .or_default()
            .push(self.current_cause.clone());
        entity.export()
    }

    pub fn destroy_entity(&mut self, entity: EntityKey) -> WorldResult {
        let entity = entity.validate(&self.entity_storage, AllowUncommitted)?;
        if self.entity_storage.is_not_committed(entity.index) {
            // component data is not deleted here, because it will be deleted at once later

            self.entities_to_commit.remove(&entity);
            self.entity_storage.delete_entity_data(entity.index);
        } else {
            self.entities_to_destroy.entry(entity).or_default().push(
                self.current_cause
                    .create_consequence("destroy_entity".to_owned()),
            )
        }
        Ok(())
    }

    pub fn entity_exists(&self, entity: EntityKey) -> bool {
        entity
            .validate(&self.entity_storage, AllowUncommitted)
            .is_ok()
    }
}
impl World {

    fn get_component_mapping_mut(
        &mut self,
        component_type: ComponentType,
    ) -> &mut HashMap<EntityIndex, ComponentDataKey> {
        self.component_mappings
            .data_by_entity_by_type
            .entry(component_type)
            .or_default()
    }
}
impl World {
    pub(crate) fn flush_entity_create_actions(&mut self) {
        for (task, causes) in mem::take(&mut self.entities_to_commit) {
            self.entity_storage.mark_committed(task.index);
            self.filter_manager.on_entity_created(task, causes);
        }
    }

    pub(crate) fn flush_component_addition(&mut self) {
        for (component_key, versions) in mem::take(&mut self.components_to_add) {
            let mut versions = versions.into_iter();

            let chosen_version = versions.next().unwrap();

            let mut all_causes = OptTinyVec::single(chosen_version.cause);
            all_causes.extend(versions.map(|it| it.cause));

            let chosen_version = self.component_data_pumps
                .get(&component_key.component_type)
                .unwrap()
                .do_move(
                    self.component_data_uncommitted.get_pool_mut(component_key.component_type).unwrap(),
                    self.component_data.get_pool_mut(component_key.component_type).unwrap(),
                    &chosen_version.data
                );

            let mapping = self
                .get_component_mapping_mut(component_key.component_type)
                .entry(component_key.entity.index)
                .or_insert(chosen_version);
            assert_eq!(
                *mapping, chosen_version,
                "attempt to mark committed as committed"
            );

            self.entity_component_index
                .add_component_type(component_key.entity.index, component_key.component_type);

            self.filter_manager.on_component_added(
                &self.entity_component_index,
                FilterComponentChange {
                    component_key,
                    causes: all_causes,
                },
            );
        }
        // deleting cancelled components (which entity or themselves was deleted at the same transaction)
        self.component_data_uncommitted.clear();
    }

    pub(crate) fn invoke_disappear_handlers(&mut self) {
        invoke_handlers(
            self.filter_manager.take_with_new_disappear_events(),
            &mut self.on_disappear,
            &mut self.current_cause,
            ComponentEventType::Disappear,
        );
    }

    pub(crate) fn invoke_appear_handlers(&mut self) {
        invoke_handlers(
            self.filter_manager.take_with_new_appear_events(),
            &mut self.on_appear,
            &mut self.current_cause,
            ComponentEventType::Appear,
        );
    }
}

enum ComponentEventType {
    Appear,
    Disappear,
}

fn invoke_handlers(
    mut filters: FilterIter<impl Iterator<Item = InternalFilterKey>>,
    handlers: &mut HashMap<InternalFilterKey, Vec<EventHandler>>,
    current_cause: &mut Cause,
    event_type: ComponentEventType,
) {
    while let Some(filter) = filters.next() {
        if let Some(handlers) = handlers.get_mut(&filter.unique_key) {
            for handler in handlers {
                let events = match event_type {
                    ComponentEventType::Appear => &mut filter.appear_events,
                    ComponentEventType::Disappear => &mut filter.disappear_events,
                };
                if let Some(events) = events {
                    for (entity, causes) in events {
                        let new_cause = current_cause.create_consequence(handler.name.to_string());
                        let prev_cause = mem::replace(current_cause, new_cause);
                        (handler.callback)(entity.export());
                        *current_cause = prev_cause;
                    }
                }
            }
        }
    }
}

impl World {
    pub(crate) fn flush_entity_destroy_actions(&mut self) {
        for (entity, causes) in mem::take(&mut self.entities_to_destroy) {
            self.entity_storage.delete_entity_data(entity.index);
            self.filter_manager.on_entity_destroyed(entity, causes);
        }
    }

    pub(crate) fn flush_component_removals(&mut self) {
        for (component_key, causes) in mem::take(&mut self.components_to_delete.after_disappear) {
            let data_key = self.component_mappings
                .data_by_entity_by_type
                .get_mut(&component_key.component_type)
                .unwrap()
                .remove(&component_key.entity.index);
            if let Some(data_key) = data_key {
                self.component_data
                    .get_pool_mut(component_key.component_type)
                    .unwrap()
                    .del(&data_key);
            }

            self.entity_component_index
                .delete_component_type(component_key.entity.index, component_key.component_type);
            self.filter_manager
                .on_component_removed(FilterComponentChange {
                    component_key,
                    causes,
                })
        }
    }

    pub(crate) fn generate_disappear_events(&mut self) {
        for (component_key, causes) in mem::take(&mut self.components_to_delete.before_disappear) {
            self.filter_manager
                .generate_disappear_events(FilterComponentChange {
                    component_key,
                    // TODO consider avoid cloning
                    causes: causes.clone(),
                });
            self.components_to_delete
                .after_disappear
                .insert(component_key, causes);
        }
    }

    pub(crate) fn schedule_destroyed_entities_component_removal(&mut self) {
        for (entity, causes) in mem::take(&mut self.entities_to_destroy) {
            for component_type in self
                .entity_component_index
                .get_component_types(entity.index)
            {
                let existing_causes = self
                    .components_to_delete
                    .before_disappear
                    .entry(ComponentKey {
                        entity,
                        component_type,
                    })
                    .or_default();
                for cause in causes.clone() {
                    existing_causes.push(cause);
                }
            }
        }
    }

    pub(crate) fn invoke_signal_handler(&mut self) {
        if let Some(signal) = self.signal_queue.signals.pop_front() {
            let manager = self.signal_managers.get_mut(&signal.payload_type).unwrap();
            manager.invoke(
                signal,
                &mut self.current_cause,
                &mut self.signal_queue,
                &mut self.signal_storage,
                &mut self.filter_manager,
            );
        }
    }
}

#[Error]
#[derive(Eq, PartialEq)]
pub enum EntityError {
    NotExists,
    NotCommitted,
    IsStale,
}
