use crate::component::ComponentType;
use crate::component::EcsComponent;
use crate::internal::world_extras::InternalEntityKey;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct ComponentKey {
    pub(crate) entity: InternalEntityKey,
    pub(crate) component_type: ComponentType,
}

impl Display for ComponentKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}>", self.entity, self.component_type)
    }
}

impl ComponentKey {
    pub(crate) fn of<T: EcsComponent>(entity: InternalEntityKey) -> ComponentKey {
        ComponentKey {
            entity,
            component_type: T::get_component_type(),
        }
    }
}
