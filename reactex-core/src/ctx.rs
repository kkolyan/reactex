// use crate::api::{FilterKey, WorldChanges};
// use crate::entity::{EntityKey, InternalEntityKey};
// use crate::mut_ref::Mut;
// use crate::world_state::WorldState;
// use crate::StaticComponentType;
//
// pub struct Ctx<'a, T = ()> {
//     pub state: &'a WorldState,
//     pub changes: &'a mut WorldChanges,
//     pub signal: &'a T,
// }
//
// impl<P> Ctx<'_, P> {
//     pub fn new_entity(&self) -> EntityKey {
//         todo!()
//     }
//
//     pub fn query(&self, filter_key: &FilterKey, mut callback: impl FnMut(EntityKey)) {
//     }
//
//     pub fn get_component<T: StaticComponentType>(&self, entity_key: EntityKey) -> Option<&T> {
//         None
//     }
//
//     pub fn get_component_for_mut<'a, T: StaticComponentType>(
//         &self,
//         entity_key: EntityKey,
//     ) -> Option<Mut<'a, T>> {
//         self.get_component(entity_key).map(|it| Mut {
//             entity: entity_key,
//             value: it,
//         })
//     }
//
//     pub fn delete_entity(&mut self, entity: EntityKey) {}
//
//     pub fn remove_component<T: StaticComponentType>(&mut self, entity: EntityKey) {}
// }
