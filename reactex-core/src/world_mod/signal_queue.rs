use crate::cause::Cause;
use crate::world_mod::signal_storage::SignalDataKey;
use crate::world_mod::world::Signal;
use std::any::{type_name, TypeId};
use std::collections::VecDeque;

#[derive(Default)]
pub struct SignalQueue {
    pub(crate) signals: VecDeque<Signal>,
}

impl SignalQueue {
    pub fn signal<T: 'static>(&mut self, data: SignalDataKey, current_cause: &Cause) {
        self.signals.push_back(Signal {
            type_name: type_name::<T>(),
            payload_type: TypeId::of::<T>(),
            data_key: data,
            cause: current_cause.clone(),
        })
    }
}
