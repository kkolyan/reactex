use crate::component::ComponentType;
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
    pub(crate) fn new(entity: InternalEntityKey, component_type: ComponentType) -> ComponentKey {
        ComponentKey {
            entity,
            component_type,
        }
    }
}
