use crate::internal::filter_manager::InternalFilterKey;
use crate::internal::signal_manager::AbstractSignalManager;
use crate::internal::signal_manager::SignalManager;
use crate::internal::world_extras::EventHandler;
use std::any::TypeId;
use std::collections::HashMap;

pub struct ImmutableWorld {
    pub(crate) signal_managers: HashMap<TypeId, Box<dyn AbstractSignalManager>>,
    pub(crate) on_appear: HashMap<InternalFilterKey, Vec<EventHandler>>,
    pub(crate) on_disappear: HashMap<InternalFilterKey, Vec<EventHandler>>,
}

impl ImmutableWorld {
    pub(crate) fn new() -> Self {
        Self {
            on_appear: Default::default(),
            on_disappear: Default::default(),
            signal_managers: Default::default(),
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
}
