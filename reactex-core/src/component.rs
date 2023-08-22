use crate::internal::world_core::COMPONENT_NAMES;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::panic::RefUnwindSafe;

pub trait EcsComponent: RefUnwindSafe + 'static {
    const INDEX: u16;
    const NAME: &'static str;

    fn get_component_type() -> ComponentType {
        ComponentType { index: Self::INDEX }
    }
}

pub const fn component_type_of<T: EcsComponent>() -> ComponentType {
    ComponentType { index: T::INDEX }
}

const fn component_type_gt(a: ComponentType, b: ComponentType) -> bool {
    a.index > b.index
}

pub const fn sort_component_types<const N: usize>(
    mut arr: [ComponentType; N],
) -> [ComponentType; N] {
    loop {
        let mut swapped = false;
        let mut i = 1;
        while i < arr.len() {
            if component_type_gt(arr[i - 1], arr[i]) {
                let left = arr[i - 1];
                let right = arr[i];
                arr[i - 1] = right;
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
    pub(crate) index: u16,
}

impl Display for ComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let guard = COMPONENT_NAMES.read().unwrap();
        let map = guard.as_ref().unwrap();
        write!(f, "{}", map.get(self).unwrap())
    }
}
