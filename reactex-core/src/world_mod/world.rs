use crate::cause::Cause;
use crate::component::ComponentType;
use crate::component::EcsComponent;
use crate::entity::EntityIndex;
use crate::entity::EntityKey;
use crate::entity::InternalEntityKey;
use crate::filter::events::FilterComponentChange;
use crate::filter::filter_desc::FilterDesc;
use crate::filter::filter_manager::FilterManager;
use crate::filter::filter_manager::InternalFilterKey;
use crate::opt_tiny_vec::OptTinyVec;
use crate::pool_pump::AbstractPoolPump;
use crate::pool_pump::SpecificPoolPump;
use crate::world_mod::component_mapping::ComponentMappingStorage;
use crate::world_mod::component_pool_manager::ComponentDataKey;
use crate::world_mod::component_pool_manager::ComponentPoolManager;
use crate::world_mod::component_pool_manager::TempComponentDataKey;
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
use justerror::Error;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;
use std::ops::Deref;
use std::sync::Mutex;

pub struct VolatileWorld {
    entity_component_index: EntityComponentIndex,
    pub(crate) entities_to_destroy: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    entities_to_commit: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    pub(crate) components_to_delete: DeleteQueue,
    pub(crate) components_to_add: HashMap<ComponentKey, OptTinyVec<ComponentAdd>>,
    pub(crate) components_to_modify: HashMap<ComponentKey, OptTinyVec<ComponentModify>>,
    pub(crate) component_data_uncommitted: ComponentPoolManager<TempComponentDataKey>,
    pub(crate) current_cause: Cause,
    pub(crate) signal_queue: SignalQueue,
    pub(crate) signal_storage: SignalStorage,
}

static COMPONENT_TYPE_REGISTRATIONS: Mutex<Vec<fn(&mut World)>> = Mutex::new(Vec::new());

pub fn register_type(registration: fn(&mut World)) {
    COMPONENT_TYPE_REGISTRATIONS
        .lock()
        .unwrap()
        .push(registration);
}

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

        pipeline::configure_pipeline(&mut world);
        world
    }

    pub fn register_component<T: EcsComponent>(&mut self) {
        self.stable.component_data.init_pool::<T>("live components");
        self.volatile
            .component_data_uncommitted
            .init_pool::<T>("temporary values");

        self.stable
            .component_data_pumps
            .entry(T::get_component_type())
            .or_insert(Box::<
                SpecificPoolPump<TempComponentDataKey, ComponentDataKey, T>,
            >::default());
    }
}

pub struct StableWorld {
    pub(crate) component_data: ComponentPoolManager<ComponentDataKey>,
    pub(crate) component_mappings: ComponentMappingStorage,
    pub(crate) filter_manager: RefCell<FilterManager>,
    pub(crate) entity_storage: RefCell<EntityStorage>,
    signal_managers: HashMap<TypeId, Box<dyn AbstractSignalManager>>,
    pub(crate) on_appear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) on_disappear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) sequence: Vec<Step>,
    component_data_pumps:
        HashMap<ComponentType, Box<dyn AbstractPoolPump<TempComponentDataKey, ComponentDataKey>>>,
}

impl StableWorld {
    fn new() -> StableWorld {
        Self {
            component_data: Default::default(),
            component_mappings: Default::default(),
            filter_manager: Default::default(),
            entity_storage: RefCell::new(EntityStorage::with_capacity(512)),
            sequence: vec![],
            on_appear: Default::default(),
            on_disappear: Default::default(),
            signal_managers: Default::default(),
            component_data_pumps: Default::default(),
        }
    }
}

impl VolatileWorld {
    pub fn signal<T: 'static>(&mut self, payload: T) {
        let mut sender = SignalSender {
            signal_queue: &mut self.signal_queue,
            current_cause: &self.current_cause,
            signal_storage: &mut self.signal_storage,
        };
        sender.signal(payload);
    }
}

impl StableWorld {
    pub(crate) fn get_signal_manager<T: 'static>(&mut self) -> &mut SignalManager<T> {
        self.signal_managers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<SignalManager<T>>::default())
            .as_any_mut()
            .try_specialize::<T>()
            .unwrap()
    }
}

impl StableWorld {
    pub fn query(&self, filter: FilterDesc, mut callback: impl FnMut(EntityKey)) {
        for matched_entity in self
            .filter_manager
            .borrow_mut()
            .get_filter(filter)
            .track_matched_entities(
                self.entity_storage.borrow().deref(),
                &self.component_mappings,
            )
            .iter()
        {
            callback(matched_entity.export());
        }
    }
}

impl VolatileWorld {
    pub fn new() -> Self {
        VolatileWorld {
            entity_component_index: EntityComponentIndex::new(512, 8),
            entities_to_destroy: Default::default(),
            entities_to_commit: Default::default(),
            components_to_delete: Default::default(),
            components_to_add: Default::default(),
            components_to_modify: Default::default(),
            component_data_uncommitted: Default::default(),
            current_cause: Cause::initial(),
            signal_queue: Default::default(),
            signal_storage: SignalStorage::new(),
        }
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
    pub(crate) fn of<T: EcsComponent>(entity: InternalEntityKey) -> ComponentKey {
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

pub struct ComponentModify {
    callback: Box<dyn FnOnce(&mut dyn Any)>,
    cause: Cause,
}

pub struct EventHandler {
    pub(crate) name: &'static str,
    pub(crate) callback: Box<dyn Fn(EntityKey, &StableWorld, &mut VolatileWorld)>,
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

impl VolatileWorld {
    pub(crate) fn modify_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        change: impl FnOnce(&mut T) + 'static,
        entity_storage: &EntityStorage,
    ) -> WorldResult {
        let entity = entity.validate(entity_storage, DenyUncommitted)?;

        self.components_to_modify
            .entry(ComponentKey::of::<T>(entity))
            .or_default()
            .push(ComponentModify {
                callback: Box::new(move |state| {
                    let state: &mut T = state.downcast_mut().unwrap();
                    change(state);
                }),
                cause: self.current_cause.clone(),
            });
        Ok(())
    }

    pub(crate) fn add_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        component: T,
        entity_storage: &EntityStorage,
    ) -> WorldResult {
        let entity = entity.validate(entity_storage, AllowUncommitted)?;

        let data = self
            .component_data_uncommitted
            .get_pool_mut(T::get_component_type())
            .specializable_mut()
            .try_specialize::<T>()
            .unwrap()
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

    pub(crate) fn remove_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        entity_storage: &EntityStorage,
    ) -> WorldResult {
        let entity = entity.validate(entity_storage, DenyUncommitted)?;

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

impl VolatileWorld {
    pub(crate) fn create_entity(
        &mut self,
        entity_storage: &mut EntityStorage,
    ) -> InternalEntityKey {
        let entity = entity_storage.new_entity();
        self.entity_component_index.add_entity(entity.index);
        self.entities_to_commit
            .entry(entity)
            .or_default()
            .push(self.current_cause.clone());
        entity
    }

    pub(crate) fn destroy_entity(
        &mut self,
        entity: EntityKey,
        entity_storage: &mut EntityStorage,
    ) -> WorldResult {
        let entity = entity.validate(entity_storage, AllowUncommitted)?;
        if entity_storage.is_not_committed(entity.index) {
            // component data is not deleted here, because it will be deleted at once later

            self.entities_to_commit.remove(&entity);
            entity_storage.delete_entity_data(entity.index);
        } else {
            self.entities_to_destroy
                .entry(entity)
                .or_default()
                .push(Cause::consequence(
                    "destroy_entity",
                    [self.current_cause.clone()],
                ))
        }
        Ok(())
    }
}

impl StableWorld {
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
        for (task, causes) in mem::take(&mut self.volatile.entities_to_commit) {
            self.stable
                .entity_storage
                .get_mut()
                .mark_committed(task.index);
            self.stable
                .filter_manager
                .get_mut()
                .on_entity_created(task, causes);
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

            self.stable.filter_manager.get_mut().on_component_added(
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
        invoke_handlers(
            ComponentEventType::Disappear,
            &mut self.stable,
            &mut self.volatile,
        );
    }

    pub(crate) fn invoke_appear_handlers(&mut self) {
        invoke_handlers(
            ComponentEventType::Appear,
            &mut self.stable,
            &mut self.volatile,
        );
    }
}

enum ComponentEventType {
    Appear,
    Disappear,
}

fn invoke_handlers(
    event_type: ComponentEventType,
    stable: &mut StableWorld,
    volatile: &mut VolatileWorld,
) {
    let filter_manager = &mut stable.filter_manager.borrow_mut();
    let filters = mem::take(match event_type {
        ComponentEventType::Appear => &mut filter_manager.with_new_appear_events,
        ComponentEventType::Disappear => &mut filter_manager.with_new_disappear_events,
    });
    let handlers = match event_type {
        ComponentEventType::Appear => &stable.on_appear,
        ComponentEventType::Disappear => &stable.on_disappear,
    };
    for filter in filters {
        let filter = filter_manager.get_filter_internal(filter);
        if let Some(handlers) = handlers.get(&filter.unique_key) {
            for handler in handlers {
                let events = match event_type {
                    ComponentEventType::Appear => &mut filter.appear_events,
                    ComponentEventType::Disappear => &mut filter.disappear_events,
                };
                let events = events.as_mut().map(mem::take);
                if let Some(events) = events {
                    for (entity, causes) in events {
                        let new_cause = Cause::consequence(handler.name, causes);
                        let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);
                        (handler.callback)(entity.export(), stable, volatile);
                        volatile.current_cause = prev_cause;
                    }
                }
            }
        }
    }
}

impl World {
    pub(crate) fn flush_entity_destroy_actions(&mut self) {
        for (entity, causes) in mem::take(&mut self.volatile.entities_to_destroy) {
            self.stable
                .entity_storage
                .get_mut()
                .delete_entity_data(entity.index);
            self.stable
                .filter_manager
                .get_mut()
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
                .get_mut()
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
            self.stable
                .filter_manager
                .get_mut()
                .generate_disappear_events(FilterComponentChange {
                    component_key,
                    // TODO consider avoid cloning
                    causes: causes.clone(),
                });
            self.volatile
                .components_to_delete
                .after_disappear
                .insert(component_key, causes);
        }
    }

    pub(crate) fn schedule_destroyed_entities_component_removal(&mut self) {
        for (entity, causes) in mem::take(&mut self.volatile.entities_to_destroy) {
            for component_type in self
                .volatile
                .entity_component_index
                .get_component_types(entity.index)
            {
                let existing_causes = self
                    .volatile
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

#[Error]
#[derive(Eq, PartialEq)]
pub enum EntityError {
    NotExists,
    NotCommitted,
    IsStale,
}

pub struct ConfigurableWorld {
    pub(crate) fetus: World,
}
