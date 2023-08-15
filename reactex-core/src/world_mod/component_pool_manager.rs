use crate::component::ComponentType;
use crate::component::StaticComponentType;
use crate::pools::AbstractPool;
use crate::pools::PoolKey;
use crate::pools::SpecificPool;
use log::info;
use std::collections::HashMap;

pub(crate) struct ComponentPoolManager<TComponentDataKey> {
    by_type: HashMap<ComponentType, Box<dyn AbstractPool<TComponentDataKey>>>,
}

impl<T> Default for ComponentPoolManager<T> {
    fn default() -> Self {
        Self {
            by_type: Default::default(),
        }
    }
}

impl ComponentPoolManager<TempComponentDataKey> {
    pub(crate) fn clear(&mut self) {
        for (_, pool) in &mut self.by_type {
            pool.clear();
        }
    }
}

impl<TComponentDataKey: PoolKey + 'static> ComponentPoolManager<TComponentDataKey> {
    pub fn init_pool<TComponent: StaticComponentType>(&mut self, name: &str) {
        info!("initialize pool {:?} with {}", name, TComponent::NAME);
        assert!(
            self.by_type
                .get(&TComponent::get_component_type())
                .is_none(),
            "attempt to init the same pool twice: {}",
            TComponent::NAME
        );
        self.by_type.insert(
            TComponent::get_component_type(),
            Box::new(SpecificPool::<TComponentDataKey, TComponent>::new()),
        );
    }
}

impl<TComponentDataKey: PoolKey> ComponentPoolManager<TComponentDataKey> {
    pub fn get_pool_mut(
        &mut self,
        component_type: ComponentType,
    ) -> &mut dyn AbstractPool<TComponentDataKey> {
        match self.by_type.get_mut(&component_type) {
            Some(option) => option.as_mut(),
            None => panic!("framework BUG: pool not initialized"),
        }
    }

    pub fn get_pool(
        &self,
        component_type: ComponentType,
    ) -> Option<&dyn AbstractPool<TComponentDataKey>> {
        self.by_type //
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
