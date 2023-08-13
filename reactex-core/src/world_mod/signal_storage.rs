use std::collections::HashMap;
use std::any::TypeId;
use crate::pools::{AbstractPool, PoolKey};

pub struct SignalStorage {
    pub payloads: HashMap<TypeId, Box<dyn AbstractPool<SignalDataKey>>>,
}

impl SignalStorage {
    pub(crate) fn new() -> SignalStorage {
        SignalStorage {
            payloads: Default::default(),
        }
    }
}

pub struct SignalDataKey(usize);

impl PoolKey for SignalDataKey {
    fn as_usize(&self) -> usize { self.0 }
    fn from_usize(value: usize) -> Self { SignalDataKey(value) }
}
