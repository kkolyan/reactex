use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::component::ComponentType;
use crate::filter::FilterDesc;
use crate::internal::entity_storage::EntityStorage;
use crate::internal::world_immutable::ImmutableWorld;
use crate::internal::world_pipeline;
use crate::internal::world_stable::StableWorld;
use crate::internal::world_volatile::VolatileWorld;

pub(crate) static COMPONENT_TYPE_REGISTRATIONS: Mutex<Vec<fn(&mut World)>> = Mutex::new(Vec::new());

pub(crate) static COMPONENT_NAMES: RwLock<Option<HashMap<ComponentType, &'static str>>> =
    RwLock::new(None);

pub(crate) static QUERIES: Mutex<Option<HashSet<FilterDesc>>> = Mutex::new(None);

pub struct World {
    pub(crate) volatile: VolatileWorld,
    pub(crate) stable: StableWorld,
    pub(crate) immutable: ImmutableWorld,
    pub(crate) entity_storage: EntityStorage,
    pub(crate) tx: u64,
}

impl World {
    pub(crate) fn new() -> Self {
        let mut world = Self {
            immutable: ImmutableWorld::new(),
            volatile: VolatileWorld::new(),
            stable: StableWorld::new(),
            entity_storage: EntityStorage::with_capacity(512),
            tx: 0,
        };
        for registration in COMPONENT_TYPE_REGISTRATIONS.lock().unwrap().iter() {
            registration(&mut world);
        }

        for filter in QUERIES.lock().unwrap().iter().flatten() {
            world.register_filter(*filter);
        }

        world_pipeline::configure_pipeline(&mut world);
        world
    }

    fn register_filter(&mut self, filter: FilterDesc) {
        self.stable
            .filter_manager
            .get_filter_mut(filter)
            .track_matched_entities(&self.entity_storage, &self.stable.component_mappings);
    }
}
