use std::ops::Deref;
use crate::entity::EntityKey;
use crate::filter::filter_desc::FilterDesc;
use crate::world_mod::signal_manager::EntitySignalHandler;
use crate::world_mod::signal_manager::GlobalSignalHandler;
use crate::world_mod::signal_sender::SignalSender;
use crate::world_mod::world::{EventHandler, World};

pub struct ConfigurableWorld {
    fetus: World,
}

impl ConfigurableWorld {

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { fetus: World::new() }
    }

    pub fn complete_configuration(self) -> World {
        self.fetus
    }

    pub fn add_global_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        callback: impl Fn(&T, &mut SignalSender) + 'static,
    ) {
        self.fetus.stable.get_signal_manager::<T>()
            .global_handlers
            .push(GlobalSignalHandler {
                name,
                callback: Box::new(callback),
            });
    }

    pub fn add_entity_signal_handler<T: 'static>(
        &mut self,
        name: &'static str,
        filter: FilterDesc,
        callback: impl Fn(&T, EntityKey, &mut SignalSender) + 'static,
    ) {
        let filter = self.fetus.stable.filter_manager.get_mut().get_filter(filter);
        filter.track_matched_entities(self.fetus.stable.entity_storage.borrow().deref(), &self.fetus.stable.component_mappings);
        let filter_key = filter.unique_key;
        self.fetus.stable.get_signal_manager::<T>()
            .handlers
            .entry(filter_key)
            .or_default()
            .push(EntitySignalHandler {
                name,
                callback: Box::new(callback),
            });
    }

    pub fn add_disappear_handler(
        &mut self,
        name: &'static str,
        filter_key: FilterDesc,
        callback: impl Fn(EntityKey) + 'static,
    ) {
        let filter = self.fetus.stable.filter_manager.get_mut().get_filter(filter_key);
        filter.track_disappear_events();
        let filter_key = filter.unique_key;
        self.fetus.stable.on_disappear
            .entry(filter_key)
            .or_default()
            .push(EventHandler {
                name,
                callback: Box::new(callback),
            });
    }

    pub fn add_appear_handler(
        &mut self,
        name: &'static str,
        filter_key: FilterDesc,
        callback: impl Fn(EntityKey) + 'static,
    ) {
        let filter = self.fetus.stable.filter_manager.get_mut().get_filter(filter_key);
        filter.track_appear_events();
        let filter_key = filter.unique_key;
        self.fetus.stable.on_appear
            .entry(filter_key)
            .or_default()
            .push(EventHandler {
                name,
                callback: Box::new(callback),
            });
    }
}
