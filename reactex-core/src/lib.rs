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
    fn get_component_type() -> ComponentType {
        ComponentType {
            type_id: TypeId::of::<Self>(),
        }
    }
}

#[macro_export]
macro_rules! ecs_component {
    ($ty:ty) => {
        impl reactex_core::StaticComponentType for $ty {
            // fn get_component_type() -> reactex_core::ComponentType {
            //     todo!()
            //     // static INDEX: usize = reactex_core::component_index::next_component_index();
            //     // reactex_core::ComponentType { index: INDEX }
            // }
        }
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ComponentType {
    type_id: TypeId,
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
