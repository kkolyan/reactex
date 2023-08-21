use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;

use crate::internal::cause::Cause;
use crate::internal::change_buffer::ChangeBuffer;
use crate::internal::entity_key_generator::TemporaryEntityKeyStorage;
use crate::Ctx;
use crate::internal::entity_storage::EntityStorage;
use crate::StableWorld;
use crate::VolatileWorld;

pub(crate) fn invoke_user_code<R>(
    volatile: &mut VolatileWorld,
    stable: &StableWorld,
    entity_storage: &mut EntityStorage,
    handler_name: &'static str,
    causes: impl IntoIterator<Item = Cause>,
    code: impl IntoIterator<Item = impl UserCode<R>>,
    mut result_handler: impl FnMut(R),
) {
    let new_cause = Cause::consequence(handler_name, causes);
    let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);
    for code in code {
        let mut changes = ChangeBuffer::new(TemporaryEntityKeyStorage::new());
        let changes_ref = RefCell::new(&mut changes);
        let result = code.execute(&changes_ref, stable, entity_storage);
        result_handler(result);
        changes.apply_to(
            volatile,
            entity_storage,
            &stable.component_mappings,
        );
    }
    volatile.current_cause = prev_cause;
}

pub(crate) trait UserCode<R = ()> {
    fn execute<'a>(self, changes: &'a RefCell<&'a mut ChangeBuffer>, stable: &'a StableWorld, entity_storage: &'a mut EntityStorage) -> R;
}

pub(crate) struct EntityEventHandlerCode {}

impl UserCode for EntityEventHandlerCode {
    fn execute<'a>(self, changes: &'a RefCell<&'a mut ChangeBuffer>, stable: &'a StableWorld, entity_storage: &'a mut EntityStorage)  {
        todo!()
    }
}

pub(crate) struct GlobalSignalHandlerCode {}

impl UserCode for GlobalSignalHandlerCode {
    fn execute<'a>(self, changes: &'a RefCell<&'a mut ChangeBuffer>, stable: &'a StableWorld, entity_storage: &'a mut EntityStorage)  {
        todo!()
    }
}

pub(crate) struct EntitySignalHandlerCode {}

impl UserCode for EntitySignalHandlerCode {
    fn execute<'a>(self, changes: &'a RefCell<&'a mut ChangeBuffer>, stable: &'a StableWorld, entity_storage: &'a mut EntityStorage)  {
        todo!()
    }
}

pub(crate) struct ExecuteOnceCode<F> {
    pub(crate) callback: F,
}

impl<R, F: FnOnce(Ctx) -> R> UserCode<R> for ExecuteOnceCode<F> {
    fn execute<'a>(self, changes: &'a RefCell<&'a mut ChangeBuffer>, stable: &'a StableWorld, entity_storage: &'a mut EntityStorage) -> R {
        let ctx = Ctx::new(&(), stable, entity_storage, changes);
        (self.callback)(ctx)
    }
}
