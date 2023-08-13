use crate::cause::Cause;
use crate::opt_tiny_vec::OptTinyVec;
use crate::world_mod::world::ComponentKey;

pub(crate) struct FilterComponentChange {
    pub component_key: ComponentKey,
    pub causes: OptTinyVec<Cause>,
}
