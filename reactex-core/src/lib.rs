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
mod typed_index_vec;

pub trait StaticComponentType: Debug + 'static {
    const INDEX: u16;
    const NAME: &'static str;

    fn get_component_type() -> ComponentType {
        ComponentType { index: Self::INDEX }
    }
}

pub const fn component_type_of<T: StaticComponentType>() -> ComponentType {
    ComponentType { index: T::INDEX }
}

const fn component_type_gt(a: ComponentType, b: ComponentType) -> bool {
    a.index > b.index
}

pub const fn sort_component_types<const N: usize>(mut arr: [ComponentType; N]) -> [ComponentType; N] {
    loop {
        let mut swapped = false;
        let mut i = 1;
        while i < arr.len() {
            if component_type_gt(arr[i-1], arr[i]) {
                let left = arr[i-1];
                let right = arr[i];
                arr[i-1] = right;
                arr[i] = left;
                swapped = true;
            }
            i += 1;
        }
        if !swapped {
            break;
        }
    }
    arr
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct ComponentType {
    index: u16,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct FilterKey {
    component_types: &'static [ComponentType],
}

impl FilterKey {
    pub const fn new(component_types: &'static [ComponentType]) -> FilterKey {
        FilterKey { component_types }
    }
}
