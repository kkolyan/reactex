use crate::cause::Cause;
use crate::world_mod::signal_queue::SignalQueue;
use crate::world_mod::signal_storage::SignalStorage;
use log::trace;
use std::any::type_name;
use std::any::TypeId;

pub struct SignalSender<'a> {
    pub(crate) signal_queue: &'a mut SignalQueue,
    pub(crate) signal_storage: &'a mut SignalStorage,
    pub(crate) current_cause: &'a Cause,
}

impl<'a> SignalSender<'a> {
    pub fn signal<T: 'static>(&mut self, payload: T) {
        trace!("enqueueing signal");
        let cause = self.current_cause;
        let data_key = self
            .signal_storage
            .payloads
            .get_mut(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("there is not handlers for signal {}", type_name::<T>()))
            .specializable_mut()
            .try_specialize::<T>()
            .unwrap()
            .add(payload);
        self.signal_queue.signal::<T>(data_key, cause);
    }
}
