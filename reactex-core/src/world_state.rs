// use crate::api::{ComponentType, ComponentTypeAware, FilterDesc, GetRef};
// use crate::mut_ref::Mut;
// use std::any::TypeId;
// use crate::entity::EntityKey;
//
// pub struct WorldState {}
//
// impl WorldState {
//     pub fn query(&self, filter_key: &FilterDesc, mut callback: impl FnMut(EntityKey)) {
//     }
//
//     pub fn get_component<T: GetRef + 'static>(&self, entity_key: EntityKey) -> Option<&T> {
//         Some(T::get())
//     }
//
//     pub fn get_component_for_mut<'a, T: GetRef + 'static>(
//         &self,
//         entity_key: EntityKey,
//     ) -> Option<Mut<'a, T>> {
//         Some(Mut {
//             entity: entity_key,
//             value: T::get(),
//         })
//     }
// }
//
// impl ComponentTypeAware for WorldState {
//     fn get_component_type<T: 'static>(&self) -> ComponentType {
//         ComponentType(TypeId::of::<T>())
//     }
// }
