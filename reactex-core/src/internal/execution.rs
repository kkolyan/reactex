use std::mem;

use crate::internal::cause::Cause;
use crate::VolatileWorld;

pub fn invoke_user_code<T>(
    volatile: &mut VolatileWorld,
    handler_name: &'static str,
    causes: impl IntoIterator<Item = Cause>,
    instances: impl IntoIterator<Item = T>,
    code: impl FnOnce(&mut VolatileWorld, T) + Sized + Copy,
) {
    let new_cause = Cause::consequence(handler_name, causes);
    let prev_cause = mem::replace(&mut volatile.current_cause, new_cause);
    for instance in instances {
        code(volatile, instance);
    }
    volatile.current_cause = prev_cause;
}
