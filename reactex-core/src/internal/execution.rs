use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::panic::catch_unwind;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;

use crate::internal::cause::Cause;
use crate::internal::change_buffer::ChangeBuffer;
use crate::internal::entity_key_generator::TemporaryEntityKeyStorage;
use crate::internal::entity_storage::EntityStorage;
use crate::Ctx;
use crate::StableWorld;
use crate::VolatileWorld;

pub(crate) fn invoke_user_code<R, P: RefUnwindSafe>(
    volatile: &mut VolatileWorld,
    stable: &StableWorld,
    entity_storage: &mut EntityStorage,
    handler_name: &'static str,
    causes: impl IntoIterator<Item = Cause>,
    code: impl IntoIterator<Item = impl Code<P, R>>,
    mut result_handler: impl FnMut(R),
    payload: &P,
) {
    let new_cause = Cause::consequence(handler_name, causes);
    let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);
    for code in code {
        let (changes, result) = catch_unwind(|| {
            let mut changes = ChangeBuffer::new(TemporaryEntityKeyStorage::new());
            let changes_ref = RefCell::new(&mut changes);
            let ctx = Ctx::new(payload, stable, entity_storage, &changes_ref);
            let result = code.invoke(ctx);
            (changes, result)
        })
        .unwrap();
        result_handler(result);
        changes.apply_to(volatile, entity_storage, &stable.component_mappings);
    }
    volatile.current_cause = prev_cause;
}

pub(crate) trait Code<P, R>: UnwindSafe {
    fn invoke(self, ctx: Ctx<P>) -> R;
}

pub(crate) struct UserCode<P, R, F>
where
    F: FnOnce(Ctx<P>) -> R,
{
    _pd: UnwindSafePhantomData<(P, R)>,
    f: F,
}

struct UnwindSafePhantomData<T> {
    pd: PhantomData<T>,
}

impl<T> Default for UnwindSafePhantomData<T> {
    fn default() -> Self {
        Self {
            pd: Default::default(),
        }
    }
}

impl<T> UnwindSafe for UnwindSafePhantomData<T> {}

impl<P, R, F> UserCode<P, R, F>
where
    F: FnOnce(Ctx<P>) -> R,
{
    pub(crate) fn new(f: F) -> Self {
        Self {
            _pd: Default::default(),
            f,
        }
    }
}

impl<P, R, F> Code<P, R> for UserCode<P, R, F>
where
    F: FnOnce(Ctx<P>) -> R,
    F: UnwindSafe,
    P: RefUnwindSafe,
{
    fn invoke(self, ctx: Ctx<P>) -> R {
        (self.f)(ctx)
    }
}
