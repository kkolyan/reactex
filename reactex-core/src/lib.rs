extern crate core;

use std::any::TypeId;
use std::fmt::Debug;

pub mod api;
pub mod component_index;
pub mod ctx;
pub mod entity;
mod entity_component_index;
mod entity_storage;
mod execution;
pub mod filter_manager;
pub mod gen;
pub mod lang;
pub mod mut_ref;
pub mod opt_tiny_vec;
pub mod optimistic_list;
mod pools;
mod signal_manager;
pub mod world;
pub mod world_state;

pub trait StaticComponentType: Debug + 'static {
    const INDEX: u16;
    const NAME: &'static str;

    fn get_component_type() -> ComponentType {
        ComponentType { index: Self::INDEX }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentType {
    index: u16,
}

#[derive(Clone)]
pub struct FilterKey {
    component_types: Vec<ComponentType>,
}

impl FilterKey {
    pub fn new(component_types: Vec<ComponentType>) -> FilterKey {
        FilterKey { component_types }
    }
}
