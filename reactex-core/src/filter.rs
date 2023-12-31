use crate::component::ComponentType;
use std::fmt::Display;
use std::fmt::Formatter;
use to_vec::ToVec;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct FilterDesc {
    pub(crate) component_types: &'static [ComponentType],
}

impl Display for FilterDesc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ecs_filter!( {} )",
            self.component_types
                .iter()
                .map(|it| format!("{}", it))
                .to_vec()
                .join(", ")
        )
    }
}

impl FilterDesc {
    pub const fn new(component_types: &'static [ComponentType]) -> FilterDesc {
        FilterDesc { component_types }
    }
}

#[macro_export]
macro_rules! __count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

#[macro_export]
macro_rules! __ecs_filter {
    ($($component_type:ident),*) => {
        {
            use $crate::__count as count;
            const COMPONENTS_SORTED: [$crate::ComponentType; count!($($component_type)*)]
                = $crate::sort_component_types(
                    [$($crate::component_type_of::<$component_type>()),*]
                );
            const FILTER_KEY: $crate::FilterDesc = $crate::FilterDesc::new(&COMPONENTS_SORTED);
            FILTER_KEY
        }
    };
}

pub use crate::__ecs_filter as ecs_filter;
