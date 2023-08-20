use crate::component::EcsComponent;
use crate::entity::Entity;
use std::marker::PhantomData;
use std::ops::Deref;

pub struct Mut<'a, TComponent> {
    pd: PhantomData<TComponent>,
    entity: Entity<'a>,
}

impl<'a, TComponent: EcsComponent> Deref for Mut<'a, TComponent> {
    type Target = TComponent;

    fn deref(&self) -> &Self::Target {
        self.entity.get::<TComponent>().unwrap()
    }
}

impl<'a, TComponent: EcsComponent> Mut<'a, TComponent> {
    pub fn try_new(entity: Entity<'a>) -> Option<Mut<'a, TComponent>> {
        entity.get::<TComponent>()?;
        Some(Mut {
            pd: Default::default(),
            entity,
        })
    }

    pub fn modify(&self, change: impl FnOnce(&mut TComponent) + 'static) {
        self.entity.modify(change);
    }
}
