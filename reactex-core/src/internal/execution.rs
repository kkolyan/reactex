use std::borrow::Cow;
use log::{error};
use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::mem;
use std::ops::AddAssign;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;

use crate::internal::cause::Cause;
use crate::internal::change_buffer::ChangeBuffer;
use crate::internal::entity_key_generator::TemporaryEntityKeyStorage;
use crate::internal::entity_storage::EntityStorage;
use crate::panic_hook::catch_unwind_detailed;
use crate::panic_hook::DetailedError;
use crate::Ctx;
use crate::StableWorld;
use crate::VolatileWorld;

#[derive(Debug)]
pub struct ExecutionResult {
    pub errors: Vec<ExecutionError>,
}

impl ExecutionResult {
    pub(crate) fn new() -> Self {
        Self { errors: vec![] }
    }
}

impl AddAssign for ExecutionResult {
    fn add_assign(&mut self, rhs: Self) {
        self.errors.extend(rhs.errors);
    }
}

#[derive(Debug)]
pub struct ExecutionError {
    pub details: DetailedError,
    pub cause: Cause,
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)?;
        write!(f, "{:?}", self.cause)?;
        Ok(())
    }
}

pub(crate) fn invoke_user_code<R, P: RefUnwindSafe>(
    volatile: &mut VolatileWorld,
    stable: &StableWorld,
    entity_storage: &mut EntityStorage,
    handler_name: &'static str,
    causes: impl IntoIterator<Item = Cause>,
    code: impl IntoIterator<Item = impl Code<P, R>>,
    mut result_handler: impl FnMut(R),
    payload: &P,
) -> ExecutionResult {
    let new_cause = Cause::consequence(handler_name, causes);
    let prev_cause = mem::replace(&mut volatile.current_cause, new_cause.clone());
    let mut result = ExecutionResult::new();
    for code in code {
        let code_result = catch_unwind_detailed(|| {
            let mut changes = ChangeBuffer::new(TemporaryEntityKeyStorage::new());
            let changes_ref = RefCell::new(&mut changes);
            let ctx = Ctx::new(payload, stable, entity_storage, &changes_ref);
            let code_result = code.invoke(ctx);
            (changes, code_result)
        });
        match code_result {
            Ok(result) => {
                let (changes, result) = result;
                result_handler(result);
                changes.apply_to(volatile, entity_storage, &stable.component_mappings);
            }
            Err(err) => {
                error!(
                    "handler {:?} failed: {}\n cause: {}",
                    handler_name, &err, &new_cause
                );
                result.errors.push(ExecutionError {
                    details: err,
                    cause: new_cause.clone(),
                });
            }
        }
    }
    volatile.current_cause = prev_cause;
    result
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
