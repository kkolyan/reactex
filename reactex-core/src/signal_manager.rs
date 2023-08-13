use crate::filter_manager::FilterManager;
use crate::filter_manager::InternalFilterKey;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::mem;

use crate::pools::AbstractPool;
use crate::pools::PoolKey;
use crate::pools::SpecificPool;
use crate::world::Cause;
use crate::world::EntitySignalCallback;
use crate::world::GlobalSignalCallback;
use crate::world::Signal;
use crate::world::SignalQueue;
use crate::world::SignalSender;

pub struct SignalDataKey(usize);

impl PoolKey for SignalDataKey {
    fn as_usize(&self) -> usize {
        self.0
    }
    fn from_usize(value: usize) -> Self {
        SignalDataKey(value)
    }
}

pub trait AbstractSignalManager {
    fn invoke(
        &mut self,
        signal: Signal,
        current_cause: &mut Cause,
        signal_queue: &mut SignalQueue,
        signal_storage: &mut SignalStorage,
        x: &mut FilterManager,
    );
    fn as_any_mut(&mut self) -> AnySignalManager;
}

pub struct AnySignalManager<'a> {
    any: &'a mut dyn Any,
}

pub struct EntitySignalHandler<T> {
    pub name: &'static str,
    pub callback: Box<EntitySignalCallback<T>>,
}

pub struct GlobalSignalHandler<T> {
    name: &'static str,
    callback: Box<GlobalSignalCallback<T>>,
}

impl<T> GlobalSignalHandler<T> {
    pub(crate) fn new(
        name: &'static str,
        callback: Box<GlobalSignalCallback<T>>,
    ) -> GlobalSignalHandler<T> {
        GlobalSignalHandler { name, callback }
    }
}

impl<'a> AnySignalManager<'a> {
    pub(crate) fn try_specialize<T: 'static>(self) -> Option<&'a mut SignalManager<T>> {
        self.any.downcast_mut::<SignalManager<T>>()
    }
}

pub(crate) struct SignalManager<T> {
    pub(crate) global_handlers: Vec<GlobalSignalHandler<T>>,
    pub(crate) handlers: HashMap<InternalFilterKey, Vec<EntitySignalHandler<T>>>,
}

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

impl<T> SignalManager<T> {
    pub(crate) fn new() -> Self {
        SignalManager {
            global_handlers: Default::default(),
            handlers: Default::default(),
        }
    }
}

impl<T: 'static> AbstractSignalManager for SignalManager<T> {
    fn invoke(
        &mut self,
        signal: Signal,
        current_cause: &mut Cause,
        signal_queue: &mut SignalQueue,
        signal_storage: &mut SignalStorage,
        filter_manager: &mut FilterManager,
    ) {
        let payload = signal_storage
            .payloads
            .get_mut(&signal.payload_type)
            .unwrap()
            .as_any_mut()
            .try_specialize::<T>()
            .unwrap()
            .del(&signal.data_key)
            .unwrap();

        for handler in &mut self.global_handlers {
            let prev_cause = mem::replace(
                current_cause,
                current_cause.create_consequence(handler.name.to_string()),
            );
            (handler.callback)(
                &payload,
                &mut SignalSender {
                    signal_queue,
                    current_cause,
                    signal_storage,
                },
            );
            *current_cause = prev_cause;
        }

        for (filter, handlers) in &mut self.handlers {
            for handler in handlers {
                if let Some(matched_entities) = &filter_manager.get_filter_internal(*filter).matched_entities {
                    let prev_cause = mem::replace(
                        current_cause,
                        current_cause.create_consequence(handler.name.to_string()),
                    );

                    for entity in matched_entities {
                        (handler.callback)(
                            &payload,
                            entity.export(),
                            &mut SignalSender {
                                signal_queue,
                                current_cause,
                                signal_storage,
                            });
                    }

                    *current_cause = prev_cause;
                }
            }
        }
    }

    fn as_any_mut(&mut self) -> AnySignalManager {
        AnySignalManager { any: self }
    }
}
