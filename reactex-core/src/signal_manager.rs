use crate::world::Cause;

pub struct SignalDataKey {}

pub trait AbstractSignalManager {
    fn invoke(&mut self, key: SignalDataKey, cause: Cause);
}

pub struct AnySignalManager {}

pub struct SignalManager {}
