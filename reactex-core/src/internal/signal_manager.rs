use crate::entity_key::EntityKey;
use crate::internal::cause::Cause;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use crate::Ctx;

use crate::internal::filter_manager::FilterManager;
use crate::internal::filter_manager::InternalFilterKey;
use crate::internal::signal_queue::SignalQueue;
use crate::internal::signal_sender::SignalSender;
use crate::internal::signal_storage::SignalStorage;
use crate::internal::world_extras::Signal;
use crate::internal::world_stable::StableWorld;
use crate::internal::world_volatile::VolatileWorld;

pub(crate) trait AbstractSignalManager {
    fn invoke(&self, signal: Signal, current_cause: &StableWorld, signal_queue: &mut VolatileWorld);
    fn as_any_mut(&mut self) -> AnySignalManager;
}

pub(crate) struct AnySignalManager<'a> {
    any: &'a mut dyn Any,
}

impl<'a> AnySignalManager<'a> {
    pub(crate) fn try_specialize<T: 'static>(self) -> Option<&'a mut SignalManager<T>> {
        self.any.downcast_mut::<SignalManager<T>>()
    }
}

pub(crate) struct EntitySignalHandler<T> {
    pub(crate) name: &'static str,
    pub(crate) callback: Box<EntitySignalCallback<T>>,
}

pub(crate) struct GlobalSignalHandler<T> {
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
            let ctx = Ctx {
                signal: &payload,
                stable,
                volatile: &RefCell::new(volatile),
            };
            (handler.callback)(ctx);
            volatile.current_cause = prev_cause;
        }

        for (filter, handlers) in &self.handlers {
            for handler in handlers {
                if let Some(matched_entities) = &stable
                    .filter_manager
                    .get_filter_by_key(*filter)
                    .matched_entities
                {
                    let new_cause =
                        Cause::consequence(handler.name, [volatile.current_cause.clone()]);
                    let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);

                    for entity in matched_entities {
                        let ctx = Ctx {
                            signal: &payload,
                            stable,
                            volatile: &RefCell::new(volatile),
                        };
                        (handler.callback)(ctx, entity.export());
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

pub(crate) type GlobalSignalCallback<T> = dyn Fn(Ctx<T>);
pub(crate) type EntitySignalCallback<T> = dyn Fn(Ctx<T>, EntityKey);
