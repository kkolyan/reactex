use crate::component::EcsComponent;
use crate::entity_key::EntityKey;
use crate::internal::cause::Cause;
use crate::internal::change_buffer::TempEntityKey;
use crate::internal::component_key::ComponentKey;
use crate::internal::component_mappings::ComponentMappingStorage;
use crate::internal::component_pool_manager::ComponentPoolManager;
use crate::internal::component_pool_manager::TempComponentDataKey;
use crate::internal::entity_component_index::EntityComponentIndex;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::entity_storage::ValidateUncommitted::AllowUncommitted;
use crate::internal::entity_storage::ValidateUncommitted::DenyUncommitted;
use crate::internal::signal_queue::SignalQueue;
use crate::internal::signal_sender::SignalSender;
use crate::internal::signal_storage::SignalStorage;

use crate::internal::world_extras::ComponentAdd;
use crate::internal::world_extras::ComponentModify;
use crate::internal::world_extras::DeleteQueue;
use crate::internal::world_extras::InternalEntityKey;
use crate::utils::opt_tiny_vec::OptTinyVec;
use crate::world_result::ComponentError;
use crate::world_result::WorldError;
use crate::world_result::WorldResult;
use crate::ComponentType;
use log::trace;
use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct VolatileWorld {
    pub(crate) entity_component_index: EntityComponentIndex,
    pub(crate) entities_to_destroy: DeleteQueue<InternalEntityKey>,
    pub(crate) entities_to_commit: HashMap<InternalEntityKey, OptTinyVec<Cause>>,
    pub(crate) components_to_delete: DeleteQueue<ComponentKey>,
    pub(crate) components_to_add: HashMap<ComponentKey, OptTinyVec<ComponentAdd>>,
    pub(crate) components_to_modify: HashMap<ComponentKey, OptTinyVec<ComponentModify>>,
    pub(crate) component_data_uncommitted: ComponentPoolManager<TempComponentDataKey>,
    pub(crate) current_cause: Cause,
    pub(crate) signal_queue: SignalQueue,
    pub(crate) signal_storage: SignalStorage,
}

impl VolatileWorld {
    pub(crate) fn new() -> Self {
        VolatileWorld {
            entity_component_index: EntityComponentIndex::new(512, 8),
            entities_to_destroy: DeleteQueue::new(),
            entities_to_commit: Default::default(),
            components_to_delete: DeleteQueue::new(),
            components_to_add: Default::default(),
            components_to_modify: Default::default(),
            component_data_uncommitted: Default::default(),
            current_cause: Cause::initial(),
            signal_queue: Default::default(),
            signal_storage: SignalStorage::new(),
        }
    }
}

impl VolatileWorld {
    pub(crate) fn modify_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        change: impl FnOnce(&mut T) + 'static,
        entity_storage: &EntityStorage,
    ) -> WorldResult {
        trace!("modify component {}<{}>", entity, T::NAME);

        let entity = entity.validate(entity_storage, DenyUncommitted)?;

        self.modify_component_internal(
            ComponentKey::new(entity, T::get_component_type()),
            Box::new(move |state| {
                let state: &mut T = state.downcast_mut().unwrap();
                change(state);
            }),
        );
        Ok(())
    }

    pub(crate) fn modify_component_internal(
        &mut self,
        component_key: ComponentKey,
        callback: Box<dyn FnOnce(&mut dyn Any)>,
    ) {
        self.components_to_modify
            .entry(component_key)
            .or_default()
            .push(ComponentModify { callback });
    }

    pub(crate) fn add_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        component: T,
        entity_storage: &EntityStorage,
    ) -> WorldResult {
        trace!("user requested to add component {}<{}>", entity, T::NAME);

        let entity = entity.validate(entity_storage, AllowUncommitted)?;

        let data = self
            .component_data_uncommitted
            .get_pool_mut(T::get_component_type())
            .specializable_mut()
            .try_specialize::<T>()
            .unwrap()
            .add(component);

        self.components_to_add
            .entry(ComponentKey::new(entity, T::get_component_type()))
            .or_default()
            .push(ComponentAdd {
                data,
                cause: self.current_cause.clone(),
            });

        Ok(())
    }

    pub(crate) fn add_component_dyn_internal(
        &mut self,
        component_key: ComponentKey,
        value: Box<dyn Any>,
    ) {
        let data = self
            .component_data_uncommitted
            .get_pool_mut(component_key.component_type)
            .add(value);

        self.components_to_add
            .entry(component_key)
            .or_default()
            .push(ComponentAdd {
                data,
                cause: self.current_cause.clone(),
            });
    }

    pub(crate) fn remove_component<T: EcsComponent>(
        &mut self,
        entity: EntityKey,
        entity_storage: &EntityStorage,
        component_mappings: &ComponentMappingStorage,
    ) -> WorldResult {
        self.remove_component_dyn(
            entity,
            T::get_component_type(),
            entity_storage,
            component_mappings,
        )
    }

    pub(crate) fn remove_component_dyn(
        &mut self,
        entity: EntityKey,
        component_type: ComponentType,
        entity_storage: &EntityStorage,
        component_mappings: &ComponentMappingStorage,
    ) -> WorldResult {
        trace!("remove component {}<{}>", entity, component_type);

        let entity = entity.validate(entity_storage, DenyUncommitted)?;

        self.remove_component_internal(
            ComponentKey::new(entity, component_type),
            component_mappings,
        )
    }

    pub(crate) fn remove_component_internal(
        &mut self,
        component_key: ComponentKey,
        component_mappings: &ComponentMappingStorage,
    ) -> WorldResult {
        // TODO that sucks, redesign it. success of remove_component shouldn't depend on the temporary state introduced of another system in transaction.
        let removed_uncommitted = self
            .components_to_add
            .remove(&ComponentKey::new(
                component_key.entity,
                component_key.component_type,
            ))
            .is_some();

        if removed_uncommitted {
            return Ok(());
        }
        if !component_mappings
            .has_component_no_validation(component_key.entity.index, component_key.component_type)
        {
            return Err(WorldError::Component(ComponentError::NotFound));
        }
        self.components_to_delete
            .before_disappear
            .entry(component_key)
            .or_default()
            .push(self.current_cause.clone());
        Ok(())
    }

    pub(crate) fn signal<T: 'static>(&mut self, payload: T) {
        let mut sender = SignalSender {
            signal_queue: &mut self.signal_queue,
            current_cause: &self.current_cause,
            signal_storage: &mut self.signal_storage,
        };
        sender.signal(payload);
    }

    pub(crate) fn create_entity(
        &mut self,
        entity_storage: &mut EntityStorage,
    ) -> InternalEntityKey {
        trace!("user requested create entity");
        let entity = entity_storage.new_entity();
        self.entity_component_index.add_entity(entity.index);
        self.entities_to_commit
            .entry(entity)
            .or_default()
            .push(self.current_cause.clone());
        trace!("entity created: {}", entity);
        entity
    }

    pub(crate) fn persist_entity(
        &mut self,
        entity: TempEntityKey,
        entity_storage: &mut EntityStorage,
    ) {
        trace!("persisting entity {:?}", entity);
        let entity = entity_storage.persist_generated(entity);
        self.entity_component_index.add_entity(entity.index);
        self.entities_to_commit
            .entry(entity)
            .or_default()
            .push(self.current_cause.clone());
    }

    pub(crate) fn destroy_entity(
        &mut self,
        entity: EntityKey,
        entity_storage: &mut EntityStorage,
    ) -> WorldResult {
        trace!("user requested destroy entity: {}", entity);
        let entity = entity.validate(entity_storage, AllowUncommitted)?;
        self.destroy_entity_internal(entity, entity_storage);
        Ok(())
    }

    pub(crate) fn destroy_entity_internal(
        &mut self,
        entity: InternalEntityKey,
        entity_storage: &mut EntityStorage,
    ) {
        if entity_storage.is_not_committed(entity.index) {
            // component data is not deleted here, because it will be deleted at once later

            // but we still need to delete component tasks right here
            self.components_to_add.retain(|it, _| it.entity != entity);

            self.entities_to_commit.remove(&entity);

            entity_storage.delete_entity_data(entity.index);
        } else {
            self.entities_to_destroy
                .before_disappear
                .entry(entity)
                .or_default()
                .push(Cause::consequence(
                    "destroy_entity",
                    [self.current_cause.clone()],
                ))
        }
    }
}
