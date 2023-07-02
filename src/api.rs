#[derive(Copy, Clone)]
pub struct ComponentType(usize);

#[derive(Copy, Clone)]
pub struct Entity {
    index: u32,
    generation: u32,
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

pub trait ComponentTypeAware {
    fn get_component_type<T>(&self) -> ComponentType;
}

pub trait WorldState: ComponentTypeAware {
    fn query(&self, filter_key: &FilterKey, callback: impl FnMut(Entity));
    fn get_component<T>(&self, entity_key: Entity) -> Option<&T>;
}

pub trait WorldWriter {
    fn new_entity(&mut self) -> Entity;
    fn destroy_entity(&mut self, entity: Entity);

    fn add_component<T>(&mut self, entity: Entity, component_state: T);
    fn update_component<T>(&mut self, entity: Entity, callback: impl FnOnce(&mut T));
    fn remove_component<T>(&mut self, entity: Entity);
    fn signal<T>(&mut self, signal: T);
}

pub trait ConfigurablePipeline<R: WorldState, W: WorldWriter, E: ExecutablePipeline>: ComponentTypeAware {
    fn add_global_signal_handler<T>(&mut self, callback: impl Fn(&T, &R, &mut W));
    fn add_entity_signal_handler<TSignal>(&mut self, filter_key: FilterKey, callback: impl Fn(&TSignal, Entity, &R, &mut W));
    fn add_entity_appear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(Entity, &R, &mut W));
    fn add_entity_disappear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(Entity, &R, &mut W));
    fn complete_configuration(self) -> E;
}

pub trait ExecutablePipeline: WorldWriter {
    fn execute_all(&mut self);
}



pub trait IntoFilterKey {
    fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey;
}

impl<A, B> IntoFilterKey for (A, B) {
    fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey {
        FilterKey::new(vec![
            storage.get_component_type::<A>(),
            storage.get_component_type::<B>(),
        ])
    }
}

impl<T> IntoFilterKey for (T, ) {
    fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey {
        FilterKey::new(vec![
            storage.get_component_type::<T>(),
        ])
    }
}