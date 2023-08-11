use crate::entity::EntityIndex;
use crate::opt_tiny_vec::OptTinyVec;
use crate::world::Cause;
use crate::world::ComponentKey;

pub struct FilterManager {}

impl FilterManager {
    pub(crate) fn generate_disappear_events(&self, _: ComponentChangeKey) {}
}

impl FilterManager {
    pub(crate) fn on_component_deleted(&self, _: ComponentKey) {}
}

impl FilterManager {
    pub(crate) fn on_entity_destroyed(&self, _: EntityIndex, _: OptTinyVec<Cause>) {}
}

impl FilterManager {
    pub(crate) fn on_component_added(&self, _: ComponentChangeKey) {}
}

pub(crate) struct ComponentChangeKey {
    pub component_key: ComponentKey,
    pub causes: OptTinyVec<Cause>,
}

impl FilterManager {
    pub(crate) fn on_entity_created(&self, _: EntityIndex, _: OptTinyVec<Cause>) {}
}

impl FilterManager {
    pub(crate) fn new() -> FilterManager {
        FilterManager {}
    }
}
