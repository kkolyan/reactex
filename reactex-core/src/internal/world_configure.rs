use crate::entity_key::EntityKey;
use crate::filter::FilterDesc;
use crate::internal::signal_manager::EntitySignalHandler;
use crate::internal::signal_manager::GlobalSignalHandler;
use crate::internal::signal_storage::SignalDataKey;
use crate::internal::world_core::World;
use crate::internal::world_extras::EventHandler;
use crate::utils::pools::SpecificPool;
use crate::Ctx;
use log::trace;
use std::any::TypeId;
use std::ops::Deref;

pub struct ConfigurableWorld {
    pub(crate) fetus: World,
}

impl World {
    pub(crate) fn add_global_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        callback: impl Fn(Ctx<T>) + 'static,
    ) {
        trace!(
            "register global signal handler '{}' for {}",
            name,
            std::any::type_name::<T>()
        );
        self.volatile
            .signal_storage
            .payloads
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
        callback: impl Fn(Ctx<T>, EntityKey) + 'static,
    ) {
        trace!(
            "register signal handler '{}' for {} and {}",
            name,
            std::any::type_name::<T>(),
            filter
        );
        self.volatile
            .signal_storage
            .payloads
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(SpecificPool::<SignalDataKey, T>::new()));
        let filter = self.stable.filter_manager.get_filter_mut(filter);
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
        callback: impl Fn(Ctx, EntityKey) + 'static,
    ) {
        let filter = self.stable.filter_manager.get_filter_mut(filter_key);
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
        callback: impl Fn(Ctx, EntityKey) + 'static,
    ) {
        let filter = self.stable.filter_manager.get_filter_mut(filter_key);
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
