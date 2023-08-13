use crate::entity::{EntityIndex, InternalEntityKey};
use crate::entity::EntityKey;
use crate::entity_component_index::EntityComponentIndex;
use crate::entity_storage::EntityStorage;
use crate::entity_storage::ValidateUncommitted::AllowUncommitted;
use crate::entity_storage::ValidateUncommitted::DenyUncommitted;
use crate::execution::ExecutionContext;
use crate::execution::Step;
use crate::execution::StepImpl;
use crate::filter_manager::{ComponentAddKey, ComponentChangeKey, ComponentRemoveKey};
use crate::filter_manager::FilterManager;
use crate::opt_tiny_vec::OptTinyVec;
use crate::pools::AbstractPool;
use crate::pools::PoolKey;
use crate::pools::SpecificPool;
use crate::signal_manager::AbstractSignalManager;
use crate::signal_manager::SignalDataKey;
use crate::world::ComponentNotFoundError::Data;
use crate::ComponentType;
use crate::FilterKey;
use crate::StaticComponentType;
use justerror::Error;
use log::trace;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;
use ComponentNotFoundError::Mapping;

pub struct World {
    entity_storage: EntityStorage,
    entity_component_index: EntityComponentIndex,
    entities_to_destroy: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    entities_to_commit: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    components_to_delete: DeleteQueue,
    components_to_add: HashMap<ComponentKey, OptTinyVec<ComponentAdd>>,
    component_data_pools: HashMap<ComponentType, Box<dyn AbstractPool<ComponentDataKey>>>,
    component_mappings: HashMap<ComponentType, HashMap<EntityIndex, ComponentDataKey>>,
    current_cause: Cause,
    filter_manager: FilterManager,
    sequence: Vec<Step>,
    on_appear: HashMap<FilterKey, Vec<EventHandler>>,
    on_disappear: HashMap<FilterKey, Vec<EventHandler>>,
    signal_managers: HashMap<TypeId, Box<dyn AbstractSignalManager>>,
    signals: VecDeque<Signal>,
}


impl World {
    pub(crate) fn get_component_types(&self, entity: InternalEntityKey) -> impl Iterator<Item=ComponentType> + '_ {
        self.entity_component_index.get_component_types(entity.index)
    }
}

impl World {
    pub(crate) fn get_all_entities(&self) -> impl Iterator<Item=InternalEntityKey> + '_ {
        self.entity_storage.get_all()
    }
}

impl World {
    pub fn query(&mut self, filter: FilterKey, mut callback: impl FnMut(EntityKey)) {
        for matched_entity in self.filter_manager.get_filter(filter).track_matched_entities().iter().copied() {
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
            component_data_pools: Default::default(),
            component_mappings: Default::default(),
            current_cause: Cause::initial(),
            filter_manager: FilterManager::new(),
            sequence: vec![],
            on_appear: Default::default(),
            on_disappear: Default::default(),
            signal_managers: Default::default(),
            signals: Default::default(),
        };
        configure_pipeline(&mut world);
        world
    }
}

#[derive(Default)]
pub struct DeleteQueue {
    before_disappear: HashMap<ComponentKey, OptTinyVec<Cause>>,
    after_disappear: HashMap<ComponentKey, OptTinyVec<Cause>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct TemporaryComponentKey {
    pub(crate) entity: EntityIndex,
    pub(crate) component_type: ComponentType,
}

impl TemporaryComponentKey {
    pub(crate) fn of<T: StaticComponentType>(entity: EntityIndex) -> TemporaryComponentKey {
        TemporaryComponentKey {
            entity,
            component_type: T::get_component_type(),
        }
    }
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
    data: ComponentDataKey,
    cause: Cause,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentDataKey {
    pub index: usize,
}

impl PoolKey for ComponentDataKey {
    fn as_usize(&self) -> usize {
        self.index
    }
    fn from_usize(value: usize) -> Self {
        ComponentDataKey { index: value }
    }
}

#[derive(Clone)]
pub struct Cause {
    inner: Rc<RefCell<CauseInner>>,
}

struct CauseInner {
    title: String,
    reasons: OptTinyVec<Cause>,
}

impl Cause {
    pub fn initial() -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title: "initial".to_string(),
                reasons: OptTinyVec::default(),
            })),
        }
    }

    pub(crate) fn create_consequence(&self, title: String) -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title,
                reasons: OptTinyVec::single(self.clone()),
            })),
        }
    }
}

pub struct EventHandler {
    name: String,
    callback: Box<dyn Fn(EntityKey)>,
}

pub struct Signal {
    payload_type: TypeId,
    data: SignalDataKey,
    cause: Cause,
}

impl World {
    pub fn execute_all(&mut self) {
        trace!("execute_all");
        let mut ctx = ExecutionContext { cursor: 0 };
        while ctx.cursor < self.sequence.len() {
            let step = &self.sequence[ctx.cursor];
            trace!("executing step: {}", step.name);
            ctx.cursor += 1;
            match step.callback {
                StepImpl::Fn(callback) => {
                    callback(self);
                }
                StepImpl::Goto {
                    condition,
                    destination_index,
                } => {
                    if condition(self) {
                        ctx.cursor = destination_index;
                    }
                }
            }
        }
    }
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
            .get_component_data_mut()
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
        let entity = entity
            .validate(&self.entity_storage, AllowUncommitted)?;

        let data = self.get_component_data_mut::<T>().add(component);

        self.components_to_add
            .entry(ComponentKey::of::<T>(entity))
            .or_default()
            .push(ComponentAdd {
                data,
                cause: self.current_cause.clone(),
            });

        self.entity_component_index
            .add_component_type(entity.index, T::get_component_type());
        Ok(())
    }

    pub fn remove_component<T: StaticComponentType>(&mut self, entity: EntityKey) -> WorldResult {
        let entity = entity
            .validate(&self.entity_storage, DenyUncommitted)?;

        let removed = self
            .components_to_add
            .remove(&ComponentKey::of::<T>(entity));
        match removed {
            Some(removed_value) => {
                for add in removed_value.iter() {
                    self.get_component_data_mut::<T>().del(&add.data);
                }
                self.components_to_add
                    .remove(&ComponentKey::of::<T>(entity));
            }
            None => {
                self.components_to_delete
                    .before_disappear
                    .entry(ComponentKey::of::<T>(entity))
                    .or_default()
                    .push(self.current_cause.clone());
            }
        }
        Ok(())
    }

    pub fn has_component<T: StaticComponentType>(&self, entity: EntityKey) -> WorldResult<bool> {
        let entity = entity
            .validate(&self.entity_storage, DenyUncommitted)?
            .index;
        Ok(self
            .component_mappings
            .get(&T::get_component_type())
            .map(|it| it.contains_key(&entity))
            .unwrap_or(false))
    }

    pub(crate) fn has_component_no_validation(
        &self,
        entity: EntityIndex,
        component_type: ComponentType,
    ) -> bool {
        self.component_mappings
            .get(&component_type)
            .map(|it| it.contains_key(&entity))
            .unwrap_or(false)
    }

    pub fn get_component<T: StaticComponentType>(
        &self,
        entity: EntityKey,
    ) -> WorldResult<Option<&T>> {
        let entity = entity
            .validate(&self.entity_storage, DenyUncommitted)?
            .index;
        Ok(self.get_component_no_validation(entity))
    }

    fn get_component_no_validation<T: StaticComponentType>(
        &self,
        entity: EntityIndex,
    ) -> Option<&T> {
        let data = self
            .component_mappings
            .get(&T::get_component_type())?
            .get(&entity)?;
        let instance = self.get_component_data::<T>()?.get(data);
        trace!("component found: {:?}", instance);
        instance
    }

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
        let entity = entity
            .validate(&self.entity_storage, AllowUncommitted)?;
        if self.entity_storage.is_not_committed(entity.index) {
            // destroy entity and attached components immediately, because they are non committed yet
            for component_type in self.entity_component_index.get_component_types(entity.index) {
                Self::remove_component_immediate(
                    &mut self.component_data_pools,
                    &mut self.component_mappings, component_type,
                    entity.index,
                )
            }
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

    fn remove_component_immediate(
        component_data_pools: &mut HashMap<ComponentType, Box<dyn AbstractPool<ComponentDataKey>>>,
        mappings: &mut HashMap<ComponentType, HashMap<EntityIndex, ComponentDataKey>>,
        component_type: ComponentType,
        entity_index: EntityIndex,
    ) {
        let table: &mut HashMap<EntityIndex, ComponentDataKey> =
            mappings.get_mut(&component_type).unwrap();
        let data = table.remove(&entity_index).unwrap();
        component_data_pools
            .get_mut(&component_type)
            .unwrap()
            .del(&data);
    }

    fn get_component_data<T: StaticComponentType>(
        &self,
    ) -> Option<&SpecificPool<ComponentDataKey, T>> {
        self.component_data_pools
            .get(&T::get_component_type())?
            .as_any()
            .try_specialize::<T>()
    }

    fn get_component_data_mut<T: StaticComponentType>(
        &mut self,
    ) -> &mut SpecificPool<ComponentDataKey, T> {
        self.component_data_pools
            .entry(T::get_component_type())
            .or_insert_with(|| Box::new(SpecificPool::<ComponentDataKey, T>::new()))
            .as_any_mut()
            .try_specialize::<T>()
            .unwrap()
    }

    fn get_component_mapping_mut(
        &mut self,
        component_type: ComponentType,
    ) -> &mut HashMap<EntityIndex, ComponentDataKey> {
        self.component_mappings.entry(component_type).or_default()
    }

    fn flush_entity_create_actions(&mut self) {
        for (task, causes) in mem::take(&mut self.entities_to_commit) {
            self.entity_storage.mark_committed(task.index);
            self.filter_manager.on_entity_created(task, causes);
        }
    }

    fn flush_component_addition(&mut self) {
        for (component_key, versions) in mem::take(&mut self.components_to_add) {
            let mut versions = versions.into_iter();
            let ComponentAdd {
                data: chosen_version,
                cause: chosen_cause,
            } = versions.next().unwrap();
            let mut causes = OptTinyVec::single(chosen_cause);
            for dropped_version in versions {
                self.component_data_pools
                    .get_mut(&component_key.component_type)
                    .unwrap()
                    .del(&dropped_version.data);
                causes.push(dropped_version.cause);
            }
            let mapping = self
                .get_component_mapping_mut(component_key.component_type)
                .entry(component_key.entity.index)
                .or_insert(chosen_version);
            assert_eq!(
                *mapping, chosen_version,
                "attempt to mark committed as committed"
            );

            self.filter_manager.on_component_added(
                |entity| self.entity_component_index.get_component_types(entity.index),
                ComponentAddKey {
                    component_key,
                    causes,
                });
        }
    }

    fn invoke_disappear_handlers(&mut self) {
        todo!()
    }

    fn flush_entity_destroy_actions(&mut self) {
        for (entity, causes) in mem::take(&mut self.entities_to_destroy) {
            self.entity_storage.delete_entity_data(entity.index);
            self.filter_manager.on_entity_destroyed(entity, causes);
        }
    }

    fn flush_component_removals(&mut self) {
        for (component_key, causes) in mem::take(&mut self.components_to_delete.after_disappear) {
            Self::remove_component_immediate(
                &mut self.component_data_pools,
                &mut self.component_mappings,
                component_key.component_type,
                component_key.entity.index,
            );

            self.entity_component_index
                .delete_component_type(component_key.entity.index, component_key.component_type);
            self.filter_manager.on_component_removed(ComponentRemoveKey {
                component_key,
                causes,
            })
        }
    }

    fn invoke_appear_handlers(&mut self) {
        todo!()
    }
    fn generate_appear_events(&mut self) {
        todo!()
    }

    fn generate_disappear_events(&mut self) {
        for (component_key, causes) in mem::take(&mut self.components_to_delete.before_disappear) {
            self.filter_manager
                .generate_disappear_events(ComponentChangeKey {
                    component_key,
                    // TODO consider avoid cloning
                    causes: causes.clone(),
                });
            self.components_to_delete
                .after_disappear
                .insert(component_key, causes);
        }
    }

    fn schedule_destroyed_entities_component_removal(&mut self) {
        for (entity, causes) in mem::take(&mut self.entities_to_destroy) {
            for component_type in self.entity_component_index.get_component_types(entity.index) {
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

    fn invoke_signal_handler(&mut self) {
        if let Some(_) = self.signals.pop_front() {
            todo!();
            // self.signal_managers[signal.Type].Invoke(signal.Index, signal.Cause);
        }
    }
}

macro_rules! step_simple_a {
    ($world:ident, $step:ident) => {
        add_step_simple($world, stringify!($step), World::$step, &mut $step);
    };
}
macro_rules! step_simple_b {
    ($world:ident, $step:ident) => {
        add_step_simple($world, stringify!($step), World::$step, &mut 0);
    };
}

fn configure_pipeline(world: &mut World) {
    let mut invoke_signal_handler = 0;
    let mut schedule_destroyed_entities_component_removal = 0;
    let mut generate_disappear_events = 0;
    let mut flush_component_addition = 0;

    step_simple_a!(world, invoke_signal_handler);
    step_simple_a!(world, schedule_destroyed_entities_component_removal);
    step_simple_a!(world, generate_disappear_events);
    step_simple_b!(world, invoke_disappear_handlers);
    step_simple_b!(world, flush_component_removals);
    add_goto(
        world,
        "check_destroyed_entities_early",
        |world| !world.entities_to_destroy.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto(
        world,
        "check_removed_components_early",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    step_simple_b!(world, flush_entity_destroy_actions);
    step_simple_b!(world, flush_entity_create_actions);
    step_simple_a!(world, flush_component_addition);
    step_simple_b!(world, invoke_appear_handlers);
    add_goto(
        world,
        "check_destroyed_entities_late",
        |world| !world.entities_to_destroy.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto(
        world,
        "check_removed_components_late",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    add_goto(
        world,
        "check_added_components",
        |world| !world.components_to_add.is_empty(),
        flush_component_addition,
    );
    add_goto(
        world,
        "check_signals",
        |world| !world.signals.is_empty(),
        invoke_signal_handler,
    );
}

fn add_step_simple(world: &mut World, name: &str, callback: fn(&mut World), index: &mut usize) {
    *index = world.sequence.len();
    world.sequence.push(Step {
        name: name.to_string(),
        callback: StepImpl::Fn(callback),
    })
}

fn add_goto(
    world: &mut World,
    name: &str,
    condition: fn(&World) -> bool,
    destination_index: usize,
) {
    world.sequence.push(Step {
        name: name.to_string(),
        callback: StepImpl::Goto {
            condition,
            destination_index,
        },
    })
}

trait StepAction {
    fn execute(world: &mut World, ctx: &mut ExecutionContext);
    fn get_name() -> &'static str;
}

struct InvokeSignalHandler;

impl StepAction for InvokeSignalHandler {
    fn execute(_world: &mut World, _ctx: &mut ExecutionContext) {}

    fn get_name() -> &'static str {
        "InvokeSignalHandler"
    }
}

#[Error]
#[derive(Eq, PartialEq)]
pub enum EntityError {
    NotExists,
    NotCommitted,
    IsStale,
}
