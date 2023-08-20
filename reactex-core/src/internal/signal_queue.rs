use crate::internal::cause::Cause;
use crate::internal::signal_storage::SignalDataKey;
use crate::internal::world_extras::Signal;
use std::any::TypeId;
use std::collections::VecDeque;

#[derive(Default)]
pub(crate) struct SignalQueue {
    pub(crate) signals: VecDeque<Signal>,
}

impl SignalQueue {
    pub(crate) fn signal<T: 'static>(&mut self, data: SignalDataKey, current_cause: &Cause) {
        self.signals.push_back(Signal {
            payload_type: TypeId::of::<T>(),
            data_key: data,
            cause: current_cause.clone(),
        })
    }
}
