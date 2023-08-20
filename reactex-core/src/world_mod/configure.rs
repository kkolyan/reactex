use std::any::TypeId;
use crate::entity::EntityKey;
use crate::filter::filter_desc::FilterDesc;
use crate::world_mod::signal_manager::EntitySignalHandler;
use crate::world_mod::signal_manager::GlobalSignalHandler;
use crate::world_mod::world::EventHandler;
use crate::world_mod::world::StableWorld;
use crate::world_mod::world::VolatileWorld;
use crate::world_mod::world::World;
use std::ops::Deref;
use log::trace;
use crate::pools::SpecificPool;
use crate::world_mod::signal_storage::SignalDataKey;

impl World {
    pub(crate) fn add_global_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        callback: impl Fn(&T, &StableWorld, &mut VolatileWorld) + 'static,
    ) {
        trace!("register global signal handler '{}' for {}", name, std::any::type_name::<T>());
        self.volatile.signal_storage.payloads
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(SpecificPool::<SignalDataKey, T>::new()));
        self.stable
            .get_signal_manager::<T>()
            .global_handlers
            .push(GlobalSignalHandler {
                name,
                callback: Box::new(callback),
            });
    }

    pub(crate) fn add_entity_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        filter: FilterDesc,
        callback: impl Fn(&T, EntityKey, &StableWorld, &mut VolatileWorld) + 'static,
    ) {
        trace!("register signal handler '{}' for {} and {}", name, std::any::type_name::<T>(), filter);
        self.volatile.signal_storage.payloads
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(SpecificPool::<SignalDataKey, T>::new()));
        let filter = self.stable.filter_manager.get_mut().get_filter(filter);
        filter.track_matched_entities(
            self.stable.entity_storage.borrow().deref(),
            &self.stable.component_mappings,
        );
        let filter_key = filter.unique_key;
        self.stable
            .get_signal_manager::<T>()
            .handlers
            .entry(filter_key)
            .or_default()
            .push(EntitySignalHandler {
                name,
                callback: Box::new(callback),
            });
    }

    pub(crate) fn add_disappear_handler(
        &mut self,
        name: &'static str,
        filter_key: FilterDesc,
        callback: impl Fn(EntityKey, &StableWorld, &mut VolatileWorld) + 'static,
    ) {
        let filter = self.stable.filter_manager.get_mut().get_filter(filter_key);
        filter.track_disappear_events();
        let filter_key = filter.unique_key;
        self.stable
            .on_disappear
            .entry(filter_key)
            .or_default()
            .push(EventHandler {
                name,
                callback: Box::new(callback),
            });
    }

    pub(crate) fn add_appear_handler(
        &mut self,
        name: &'static str,
        filter_key: FilterDesc,
        callback: impl Fn(EntityKey, &StableWorld, &mut VolatileWorld) + 'static,
    ) {
        let filter = self.stable.filter_manager.get_mut().get_filter(filter_key);
        filter.track_appear_events();
        let filter_key = filter.unique_key;
        self.stable
            .on_appear
            .entry(filter_key)
            .or_default()
            .push(EventHandler {
                name,
                callback: Box::new(callback),
            });
    }
}
