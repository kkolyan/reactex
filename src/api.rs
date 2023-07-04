use std::any::TypeId;

#[derive(Copy, Clone)]
pub struct ComponentType(TypeId);

#[derive(Copy, Clone)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
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
    fn get_component_type<T: 'static>(&self) -> ComponentType;
}

pub struct WorldState {

}

pub trait GetRef {
    fn get() -> &'static Self;
}

impl WorldState {
    pub fn query(&self, filter_key: &FilterKey, mut callback: impl FnMut(Entity)) {
        callback(Entity { index: 0, generation: 0 })
    }
    pub fn get_component<T: GetRef + 'static>(&self, entity_key: Entity) -> Option<&T> {
        Some(T::get())
    }
}

impl ComponentTypeAware for WorldState {
    fn get_component_type<T: 'static>(&self) -> ComponentType {
        ComponentType(TypeId::of::<T>())
    }
}


pub struct WorldChanges {}

impl WorldChanges {
    pub fn new_entity(&mut self) -> Entity {
        todo!()
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        todo!()
    }

    pub fn add_component<T>(&mut self, entity: Entity, component_state: T) {
        todo!()
    }

    pub fn update_component<T>(&mut self, entity: Entity, callback: impl FnOnce(&mut T)) {
        todo!()
    }

    pub fn remove_component<T>(&mut self, entity: Entity) {
        todo!()
    }

    pub fn signal<T>(&mut self, signal: T) {
        todo!()
    }
}

pub trait ComponentWriter {}

pub struct GlobalCtx<'a> {
    pub state: &'a WorldState,
    pub changes: &'a WorldChanges,
}

pub struct EntityCtx<'a> {
    pub state: &'a WorldState,
    pub changes: &'a mut WorldChanges,
    pub entity: Entity,
}

pub struct Mut<'a, T> {
    entity: Entity,
    pub value: &'a T
}

pub struct ConfigurablePipeline {}


impl ConfigurablePipeline {
    pub fn register_module(&mut self, module: &Module) {
        for task in module.tasks.iter() {
            // let x = task.deref();
            // x.action.deref()(self);
        }
    }
}


pub struct Module {
    tasks: Vec<Task>
}

struct Task {
    action: Box<dyn FnOnce(&mut ConfigurablePipeline) + Send + Sync>
}

impl Module {
    pub const fn new() -> Module {
        Module { tasks: vec![] }
    }

    pub fn register_later<T: Fn(&mut ConfigurablePipeline) + 'static>(&mut self, f: T) {
        // self.tasks.push(Task {
        //     action: Box::new(move |pipeline| {
        //         f(pipeline);
        //     }),
        // });
    }
}

impl ConfigurablePipeline {
    pub fn new() -> Self {
        ConfigurablePipeline {}
    }
    pub fn add_global_signal_handler<T>(&mut self, callback: impl Fn(&T, GlobalCtx)) {todo!()}
    pub fn add_entity_signal_handler<TSignal>(&mut self, filter_key: FilterKey, callback: impl Fn(&TSignal, EntityCtx)) {todo!()}
    pub fn add_entity_appear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(EntityCtx)) {todo!()}
    pub fn add_entity_disappear_handler<T>(&mut self, filter_key: FilterKey, callback: impl Fn(EntityCtx)) {todo!()}
    pub fn complete_configuration(self) -> ExecutablePipeline {
        todo!()
    }
}

impl ComponentTypeAware for ConfigurablePipeline {
    fn get_component_type<T>(&self) -> ComponentType {
        todo!()
    }
}

pub struct ExecutablePipeline {}

impl ExecutablePipeline {
    pub fn execute_all(&mut self) {
        todo!()
    }

    pub fn signal<T>(&mut self, signal: T) {
        todo!()
    }
}


pub trait IntoFilterKey {
    fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey;
}

impl<A: 'static, B: 'static> IntoFilterKey for (A, B) {
    fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey {
        FilterKey::new(vec![
            storage.get_component_type::<A>(),
            storage.get_component_type::<B>(),
        ])
    }
}

impl<T: 'static> IntoFilterKey for (T, ) {
    fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey {
        FilterKey::new(vec![
            storage.get_component_type::<T>(),
        ])
    }
}

impl ComponentTypeAware for EntityCtx<'_> {
    fn get_component_type<T: 'static>(&self) -> ComponentType {
        self.state.get_component_type::<T>()
    }
}