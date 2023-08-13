use std::any::TypeId;
use std::collections::VecDeque;
use crate::cause::Cause;
use crate::world_mod::world::Signal;
use crate::world_mod::signal_storage::SignalDataKey;

#[derive(Default)]
pub struct SignalQueue {
    pub(crate) signals: VecDeque<Signal>,
}

impl SignalQueue {
    pub fn signal<T: 'static>(&mut self, data: SignalDataKey, current_cause: &Cause) {
        self.signals.push_back(Signal {
            payload_type: TypeId::of::<T>(),
            data_key: data,
            cause: current_cause.clone(),
        })
    }
}
