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
use crate::world_mod::world::StableWorld;
use crate::world_mod::world::VolatileWorld;

pub(crate) trait AbstractSignalManager {
    fn invoke(&self, signal: Signal, current_cause: &StableWorld, signal_queue: &mut VolatileWorld);
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
    fn invoke(&self, signal: Signal, stable: &StableWorld, volatile: &mut VolatileWorld) {
        let payload = volatile
            .signal_storage
            .payloads
            .get_mut(&signal.payload_type)
            .unwrap()
            .specializable_mut()
            .try_specialize::<T>()
            .unwrap()
            .del_and_get(&signal.data_key)
            .unwrap();

        for handler in &self.global_handlers {
            let new_cause = Cause::consequence(handler.name, [volatile.current_cause.clone()]);
            let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);
            (handler.callback)(&payload, stable, volatile);
            volatile.current_cause = prev_cause;
        }

        for (filter, handlers) in &self.handlers {
            for handler in handlers {
                if let Some(matched_entities) = &stable
                    .filter_manager
                    .borrow_mut()
                    .get_filter_internal(*filter)
                    .matched_entities
                {
                    let new_cause =
                        Cause::consequence(handler.name, [volatile.current_cause.clone()]);
                    let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);

                    for entity in matched_entities {
                        (handler.callback)(&payload, entity.export(), stable, volatile);
                    }

                    volatile.current_cause = prev_cause;
                }
            }
        }
    }

    fn as_any_mut(&mut self) -> AnySignalManager {
        AnySignalManager { any: self }
    }
}

pub type GlobalSignalCallback<T> = dyn Fn(&T, &StableWorld, &mut VolatileWorld);
pub type EntitySignalCallback<T> = dyn Fn(&T, EntityKey, &StableWorld, &mut VolatileWorld);
