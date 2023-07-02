#[derive(Copy, Clone)]
pub struct ComponentType(usize);

#[derive(Copy, Clone)]
pub struct Entity {
    index: usize,
    generation: usize,
}

#[derive(Clone)]
pub struct FilterKey {
    component_types: Vec<ComponentType>,
}

pub trait WorldState {
    fn query(&self, filter_key: &FilterKey, callback: impl FnMut(Entity));
    fn get_component<T>(&self, entity_key: Entity) -> Option<&T>;
    fn get_component_type<T>() -> ComponentType;
}

pub trait WorldWriter {
    fn new_entity(&mut self) -> Entity;
    fn destroy_entity(&mut self, entity: Entity);

    fn add_component<T>(&mut self, entity: Entity, component_state: T);
    fn update_component<T>(&mut self, entity: Entity, callback: impl FnOnce(&mut T));
    fn remove_component<T>(&mut self, entity: Entity);
    fn signal<T>(&mut self, signal: T);
}

pub trait ConfigurablePipeline<R: WorldState, W: WorldWriter> {
    fn add_signal_handler<T>(&mut self, callback: impl Fn(&T, &R, &mut W));
    fn add_entity_signal_handler<T>(&mut self, callback: impl Fn(Entity, &T, &R, &mut W));
    fn add_entity_appear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(Entity, &R, &mut W));
    fn add_entity_disappear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(Entity, &R, &mut W));
    fn complete_configuration<T: ExecutablePipeline>(self) -> T;
}

pub trait ExecutablePipeline {
    fn execute_all(&mut self);
}