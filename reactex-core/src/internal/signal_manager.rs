use crate::entity_key::EntityKey;
use crate::internal::cause::Cause;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::execution::invoke_user_code;
use crate::internal::execution::UserCode;
use crate::Ctx;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::internal::filter_manager::InternalFilterKey;
use crate::internal::world_extras::Signal;
use crate::internal::world_stable::StableWorld;
use crate::internal::world_volatile::VolatileWorld;

pub(crate) trait AbstractSignalManager {
    fn invoke(
        &self,
        signal: Signal,
        stable: &mut StableWorld,
        volatile: &mut VolatileWorld,
        entity_storage: &mut EntityStorage,
    );
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

impl<T: RefUnwindSafe + UnwindSafe + 'static> AbstractSignalManager for SignalManager<T> {
    fn invoke(
        &self,
        signal: Signal,
        stable: &mut StableWorld,
        volatile: &mut VolatileWorld,
        entity_storage: &mut EntityStorage,
    ) {
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
            invoke_user_code(
                volatile,
                stable,
                entity_storage,
                handler.name,
                [signal.cause.clone()],
                [UserCode::new(|ctx| {
                    (handler.callback)(ctx);
                })],
                |_| {},
                &payload,
            );
        }

        for (filter, handlers) in &self.handlers {
            for handler in handlers {
                if let Some(matched_entities) = &stable
                    .filter_manager
                    .get_filter_by_key(*filter)
                    .matched_entities
                {
                    invoke_user_code(
                        volatile,
                        stable,
                        entity_storage,
                        handler.name,
                        [signal.cause.clone()],
                        matched_entities.iter().map(|entity| {
                            UserCode::new(|ctx| (handler.callback)(ctx, entity.export()))
                        }),
                        |_| {},
                        &payload,
                    );
                }
            }
        }
    }

    fn as_any_mut(&mut self) -> AnySignalManager {
        AnySignalManager { any: self }
    }
}

pub(crate) type GlobalSignalCallback<T> = dyn Fn(Ctx<T>) + RefUnwindSafe;
pub(crate) type EntitySignalCallback<T> = dyn Fn(Ctx<T>, EntityKey) + RefUnwindSafe;
