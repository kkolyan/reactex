use crate::component::EcsComponent;
use crate::filter::FilterDesc;
use crate::internal::component_pool_manager::ComponentDataKey;
use crate::internal::component_pool_manager::TempComponentDataKey;
use crate::internal::world_core::COMPONENT_NAMES;
use crate::internal::world_core::COMPONENT_TYPE_REGISTRATIONS;
use crate::internal::world_core::QUERIES;
use crate::utils::pool_pump::SpecificPoolPump;
use crate::World;
use std::collections::HashMap;
use std::collections::HashSet;

impl World {
    pub fn register_component<T: EcsComponent>(&mut self) {
        let mut guard = COMPONENT_NAMES.write().unwrap();
        if guard.is_none() {
            *guard = Some(HashMap::new());
        }
        guard
            .as_mut()
            .unwrap()
            .entry(T::get_component_type())
            .or_insert(T::NAME);

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

    pub fn register_type(registration: fn(&mut World)) {
        COMPONENT_TYPE_REGISTRATIONS
            .lock()
            .unwrap()
            .push(registration);
    }

    pub fn register_query(filter: FilterDesc) {
        QUERIES
            .lock()
            .unwrap()
            .get_or_insert_with(HashSet::new)
            .insert(filter);
    }
}
