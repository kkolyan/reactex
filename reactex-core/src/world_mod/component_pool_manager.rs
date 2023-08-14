use std::collections::HashMap;
use crate::component::{ComponentType, StaticComponentType};
use crate::pools::{AbstractPool, PoolKey, SpecificPool};


pub(crate) struct ComponentPoolManager<TComponentDataKey> {
    by_type: HashMap<ComponentType, Box<dyn AbstractPool<TComponentDataKey>>>
}

impl <T> Default for ComponentPoolManager<T> {
    fn default() -> Self {
        Self { by_type: Default::default() }
    }
}

impl ComponentPoolManager<TempComponentDataKey> {
    pub(crate) fn clear(&mut self) {
        for (_, pool) in &mut self.by_type {
            pool.clear();
        }
    }
}

impl <TComponentDataKey: PoolKey + 'static> ComponentPoolManager<TComponentDataKey> {
    pub fn get_pool_or_create<TComponent: StaticComponentType>(
        &mut self,
    ) -> &mut SpecificPool<TComponentDataKey, TComponent> {
        self.by_type
            .entry(TComponent::get_component_type())
            .or_insert_with(|| Box::new(SpecificPool::<TComponentDataKey, TComponent>::new()))
            .as_any_mut()
            .try_specialize::<TComponent>()
            .unwrap()
    }
}
impl <TComponentDataKey: PoolKey> ComponentPoolManager<TComponentDataKey> {

    pub fn get_pool_mut(&mut self, component_type: ComponentType) -> Option<&mut dyn AbstractPool<TComponentDataKey>>
    {
        let option = self.by_type
            .get_mut(&component_type);
        match option {
            Some(option) => Some(option.as_mut()),
            None => None,
        }
    }

    pub fn get_pool(&self, component_type: ComponentType) -> Option<&dyn AbstractPool<TComponentDataKey>> {
        self.by_type
            .get(&component_type)
            .map(|it| it.as_ref())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentDataKey {
    pub index: usize,
}

impl PoolKey for ComponentDataKey {
    fn as_usize(&self) -> usize {
        self.index
    }
    fn from_usize(value: usize) -> Self {
        Self { index: value }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TempComponentDataKey {
    pub index: usize,
}

impl PoolKey for TempComponentDataKey {
    fn as_usize(&self) -> usize {
        self.index
    }
    fn from_usize(value: usize) -> Self {
        Self { index: value }
    }
}
