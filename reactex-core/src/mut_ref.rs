use crate::entity::EntityKey;
use std::ops::Deref;

pub struct Mut<'a, T> {
    pub(crate) entity: EntityKey,
    pub(crate) value: &'a T,
}

impl<'a, T> Mut<'a, T> {
    pub fn modify(&self, f: impl FnOnce(&mut T)) {}
}

impl<'a, T> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
