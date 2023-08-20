use crate::component::ComponentType;
use crate::component::EcsComponent;
use crate::entity_key::EntityKey;
use crate::filter::FilterDesc;
use crate::internal::component_mappings::ComponentMappingStorage;
use crate::internal::component_pool_manager::ComponentDataKey;
use crate::internal::component_pool_manager::ComponentPoolManager;
use crate::internal::component_pool_manager::TempComponentDataKey;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::entity_storage::ValidateUncommitted::AllowUncommitted;
use crate::internal::entity_storage::ValidateUncommitted::DenyUncommitted;
use crate::internal::filter_manager::FilterManager;
use crate::internal::filter_manager::InternalFilterKey;
use crate::internal::signal_manager::AbstractSignalManager;
use crate::internal::signal_manager::SignalManager;
use crate::internal::world_extras::EntityIndex;
use crate::internal::world_extras::EventHandler;
use crate::internal::world_pipeline::PipelineStep;
use crate::utils::pool_pump::AbstractPoolPump;
use crate::utils::pools::AbstractPool;
use crate::utils::pools::SpecificPool;
use crate::world_result::WorldResult;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

pub struct StableWorld {
    pub(crate) entity_storage: RefCell<EntityStorage>,

    pub(crate) component_data: ComponentPoolManager<ComponentDataKey>,
    pub(crate) component_mappings: ComponentMappingStorage,
    pub(crate) filter_manager: FilterManager,
    pub(crate) signal_managers: HashMap<TypeId, Box<dyn AbstractSignalManager>>,
    pub(crate) on_appear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) on_disappear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) sequence: Vec<PipelineStep>,
    pub(crate) component_data_pumps:
        HashMap<ComponentType, Box<dyn AbstractPoolPump<TempComponentDataKey, ComponentDataKey>>>,
}

impl StableWorld {
    pub(crate) fn new() -> StableWorld {
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

    pub(crate) fn get_signal_manager<T: 'static>(&mut self) -> &mut SignalManager<T> {
        self.signal_managers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<SignalManager<T>>::default())
            .as_any_mut()
            .try_specialize::<T>()
            .unwrap()
    }

    pub(crate) fn query(&self, filter: FilterDesc, mut callback: impl FnMut(EntityKey)) {
        for matched_entity in self
            .filter_manager
            .get_filter_unmut(filter)
            .matched_entities
            .as_ref()
            .unwrap_or_else(|| panic!("query is not initialized: {}", filter))
            .iter()
        {
            callback(matched_entity.export());
        }
    }

    pub(crate) fn get_component_mapping_mut(
        &mut self,
        component_type: ComponentType,
    ) -> &mut HashMap<EntityIndex, ComponentDataKey> {
        self.component_mappings
            .data_by_entity_by_type
            .entry(component_type)
            .or_default()
    }

    pub(crate) fn get_component<T: EcsComponent>(
        &self,
        entity: EntityKey,
    ) -> WorldResult<Option<&T>> {
        let entity = entity
            .validate(self.entity_storage.borrow().deref(), DenyUncommitted)?
            .index;
        Ok(self.get_component_no_validation(entity))
    }

    pub(crate) fn get_component_no_validation<T: EcsComponent>(&self, entity: EntityIndex) -> Option<&T> {
        let data = self
            .component_mappings
            .data_by_entity_by_type
            .get(&T::get_component_type())?
            .get(&entity)?;
        let instance = self.get_component_data::<T>()?.get(data);
        // trace!("component found: {:?}", instance);
        instance
    }

    pub(crate) fn get_component_data<T: EcsComponent>(
        &self,
    ) -> Option<&SpecificPool<ComponentDataKey, T>> {
        self.component_data
            .get_pool(T::get_component_type())?
            .specializable()
            .try_specialize::<T>()
    }
    pub(crate) fn has_component<T: EcsComponent>(&self, entity: EntityKey) -> WorldResult<bool> {
        let entity = entity
            .validate(self.entity_storage.borrow().deref(), DenyUncommitted)?
            .index;
        Ok(self
            .component_mappings
            .data_by_entity_by_type
            .get(&T::get_component_type())
            .map(|it| it.contains_key(&entity))
            .unwrap_or(false))
    }

    pub(crate) fn entity_exists(&self, entity: EntityKey) -> bool {
        entity
            .validate(self.entity_storage.borrow().deref(), AllowUncommitted)
            .is_ok()
    }
}
