// use crate::ctx::Ctx;
// use crate::entity::EntityKey;
// use crate::mut_ref::Mut;
//
// pub trait ComponentTypeAware {
//     fn get_component_type<T: 'static>(&self) -> ComponentType;
// }
//
// pub trait GetRef {
//     fn get() -> &'static Self;
// }
//
// pub struct WorldChanges {}
//
// impl WorldChanges {
//     pub fn modify<T>(&mut self, instance: &Mut<T>, f: impl FnOnce(&mut T)) {}
//     pub fn new_entity(&mut self) -> EntityKey {
//         todo!()
//     }
//
//     pub fn destroy_entity(&mut self, entity: EntityKey) {
//         todo!()
//     }
//
//     pub fn add_component<T>(&mut self, entity: EntityKey, component_state: T) {
//         todo!()
//     }
//
//     pub fn remove_component<T>(&mut self, entity: EntityKey) {
//         todo!()
//     }
//
//     pub fn signal<T>(&mut self, signal: T) {
//         todo!()
//     }
// }
//
// pub struct ConfigurablePipeline {}
//
// impl ConfigurablePipeline {
//     pub fn register_module(&mut self, module: &Module) {
//         for task in module.tasks.iter() {
//             // let x = task.deref();
//             // x.action.deref()(self);
//         }
//     }
// }
//
// pub struct Module {
//     tasks: Vec<Task>,
// }
//
// struct Task {
//     action: Box<dyn FnOnce(&mut ConfigurablePipeline) + Send + Sync>,
// }
//
// impl Module {
//     pub const fn new() -> Module {
//         Module { tasks: vec![] }
//     }
//
//     pub fn register_later<T: Fn(&mut ConfigurablePipeline) + 'static>(&mut self, f: T) {
//         // self.tasks.push(Task {
//         //     action: Box::new(move |pipeline| {
//         //         f(pipeline);
//         //     }),
//         // });
//     }
// }
//
// impl ConfigurablePipeline {
//     pub fn new() -> Self {
//         ConfigurablePipeline {}
//     }
//     pub fn add_global_signal_handler<TSignal>(&mut self, callback: impl Fn(Ctx<TSignal>)) {
//         todo!()
//     }
//     pub fn add_entity_signal_handler<TSignal>(
//         &mut self,
//         filter_key: FilterKey,
//         callback: impl Fn(Ctx<TSignal>, EntityKey),
//     ) {
//         todo!()
//     }
//     pub fn add_entity_appear_handler(
//         &mut self,
//         filter_key: FilterKey,
//         callback: impl Fn(Ctx, EntityKey),
//     ) {
//         todo!()
//     }
//     pub fn add_entity_disappear_handler(
//         &mut self,
//         filter_key: FilterKey,
//         callback: impl Fn(Ctx, EntityKey),
//     ) {
//         todo!()
//     }
//     pub fn complete_configuration(self) -> ExecutablePipeline {
//         todo!()
//     }
// }
//
// impl ComponentTypeAware for ConfigurablePipeline {
//     fn get_component_type<T>(&self) -> ComponentType {
//         todo!()
//     }
// }
//
// pub struct ExecutablePipeline {}
//
// impl ExecutablePipeline {
//     pub fn execute_all(&mut self) {
//         todo!()
//     }
//
//     pub fn signal<T>(&mut self, signal: T) {
//         todo!()
//     }
// }
//
// pub trait IntoFilterKey {
//     fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey;
// }
//
// impl<A: 'static, B: 'static> IntoFilterKey for (A, B) {
//     fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey {
//         FilterKey::new(vec![
//             storage.get_component_type::<A>(),
//             storage.get_component_type::<B>(),
//         ])
//     }
// }
//
// impl<T: 'static> IntoFilterKey for (T, ) {
//     fn create_filter_key(storage: &impl ComponentTypeAware) -> FilterKey {
//         FilterKey::new(vec![storage.get_component_type::<T>()])
//     }
// }
