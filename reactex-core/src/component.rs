use std::fmt::Debug;

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
