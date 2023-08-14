use crate::cause::Cause;
use crate::entity::EntityKey;
use std::any::Any;
use std::collections::HashMap;
use std::mem;

use crate::filter::filter_manager::FilterManager;
use crate::filter::filter_manager::InternalFilterKey;
use crate::world_mod::signal_queue::SignalQueue;
use crate::world_mod::signal_sender::SignalSender;
use crate::world_mod::signal_storage::SignalStorage;
use crate::world_mod::world::Signal;

pub(crate) trait AbstractSignalManager {
    fn invoke(
        &mut self,
        signal: Signal,
        current_cause: &mut Cause,
        signal_queue: &mut SignalQueue,
        signal_storage: &mut SignalStorage,
        filter_manager: &mut FilterManager,
    );
    fn as_any_mut(&mut self) -> AnySignalManager;
}

pub struct AnySignalManager<'a> {
    any: &'a mut dyn Any,
}

impl<'a> AnySignalManager<'a> {
    pub(crate) fn try_specialize<T: 'static>(self) -> Option<&'a mut SignalManager<T>> {
        self.any.downcast_mut::<SignalManager<T>>()
    }
}

pub struct EntitySignalHandler<T> {
    pub name: &'static str,
    pub callback: Box<EntitySignalCallback<T>>,
}

pub struct GlobalSignalHandler<T> {
    pub(crate) name: &'static str,
    pub(crate) callback: Box<GlobalSignalCallback<T>>,
}

pub(crate) struct SignalManager<T> {
    pub(crate) global_handlers: Vec<GlobalSignalHandler<T>>,
    pub(crate) handlers: HashMap<InternalFilterKey, Vec<EntitySignalHandler<T>>>,
}

impl<T> Default for SignalManager<T> {
    fn default() -> Self {
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
                if let Some(matched_entities) =
                    &filter_manager.get_filter_internal(*filter).matched_entities
                {
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
                            },
                        );
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

pub type GlobalSignalCallback<T> = dyn Fn(&T, &mut SignalSender);
pub type EntitySignalCallback<T> = dyn Fn(&T, EntityKey, &mut SignalSender);
