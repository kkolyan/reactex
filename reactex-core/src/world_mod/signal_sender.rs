use std::any::TypeId;
use crate::cause::Cause;
use crate::pools::SpecificPool;
use crate::world_mod::signal_queue::SignalQueue;
use crate::world_mod::signal_storage::{SignalDataKey, SignalStorage};

pub struct SignalSender<'a> {
    pub(crate) signal_queue: &'a mut SignalQueue,
    pub(crate) signal_storage: &'a mut SignalStorage,
    pub(crate) current_cause: &'a Cause,
}

impl<'a> SignalSender<'a> {
    pub fn signal<T: 'static>(&mut self, payload: T) {
        let cause = self.current_cause;
        let data_key = self
            .signal_storage
            .payloads
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(SpecificPool::<SignalDataKey, T>::new()))
            .as_any_mut()
            .try_specialize::<T>()
            .unwrap()
            .add(payload);
        self.signal_queue.signal::<T>(data_key, cause);
    }
}
