use crate::api::*;

pub struct StubState;
pub struct StubWriter;

pub struct StubPipelineFactory {}
pub struct StubPipeline {}

impl StubPipelineFactory {
    pub fn new() -> StubPipelineFactory {
        todo!()
    }
}

impl WorldWriter for StubPipeline {
    fn new_entity(&mut self) -> Entity {
        todo!()
    }

    fn destroy_entity(&mut self, entity: Entity) {
        todo!()
    }

    fn add_component<T>(&mut self, entity: Entity, component_state: T) {
        todo!()
    }

    fn update_component<T>(&mut self, entity: Entity, callback: impl FnOnce(&mut T)) {
        todo!()
    }

    fn remove_component<T>(&mut self, entity: Entity) {
        todo!()
    }

    fn signal<T>(&mut self, signal: T) {
        todo!()
    }
}

impl ExecutablePipeline for StubPipeline {
    fn execute_all(&mut self) {
        todo!()
    }
}

impl ComponentTypeAware for StubState {
    fn get_component_type<T>(&self) -> ComponentType {
        todo!()
    }
}

impl ComponentTypeAware for StubPipelineFactory {
    fn get_component_type<T>(&self) -> ComponentType {
        todo!()
    }
}

impl WorldState for StubState {
    fn query(&self, filter_key: &FilterKey, callback: impl FnMut(Entity)) {
        todo!()
    }

    fn get_component<T>(&self, entity_key: Entity) -> Option<&T> {
        todo!()
    }
}

impl WorldWriter for StubWriter {
    fn new_entity(&mut self) -> Entity {
        todo!()
    }

    fn destroy_entity(&mut self, entity: Entity) {
        todo!()
    }

    fn add_component<T>(&mut self, entity: Entity, component_state: T) {
        todo!()
    }

    fn update_component<T>(&mut self, entity: Entity, callback: impl FnOnce(&mut T)) {
        todo!()
    }

    fn remove_component<T>(&mut self, entity: Entity) {
        todo!()
    }

    fn signal<T>(&mut self, signal: T) {
        todo!()
    }
}

impl ConfigurablePipeline<StubState, StubWriter, StubPipeline> for StubPipelineFactory {
    fn add_global_signal_handler<T>(&mut self, callback: impl Fn(&T, &StubState, &mut StubWriter)) {
        todo!()
    }

    fn add_entity_signal_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(&T, Entity, &StubState, &mut StubWriter)) {
        todo!()
    }


    fn add_entity_appear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(Entity, &StubState, &mut StubWriter)) {
        todo!()
    }

    fn add_entity_disappear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(Entity, &StubState, &mut StubWriter)) {
        todo!()
    }

    fn complete_configuration(self) -> StubPipeline {
        todo!()
    }
}