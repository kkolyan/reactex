use crate::utils::pools::AbstractPool;
use crate::utils::pools::PoolKey;
use std::any::TypeId;
use std::collections::HashMap;

pub(crate) struct SignalStorage {
    pub(crate) payloads: HashMap<TypeId, Box<dyn AbstractPool<SignalDataKey>>>,
}

impl SignalStorage {
    pub(crate) fn new() -> SignalStorage {
        SignalStorage {
            payloads: Default::default(),
        }
    }
}

pub(crate) struct SignalDataKey(usize);

impl PoolKey for SignalDataKey {
    fn as_usize(&self) -> usize {
        self.0
    }
    fn from_usize(value: usize) -> Self {
        SignalDataKey(value)
    }
}
