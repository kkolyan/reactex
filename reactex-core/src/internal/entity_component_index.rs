use crate::component::ComponentType;
use crate::component::EcsComponent;
use crate::internal::world_extras::EntityIndex;
use crate::utils::lang::boxed_slice;

pub(crate) struct EntityComponentIndex {
    component_count: Box<[u16]>,
    component_types: Box<[ComponentType]>,
    component_types_width: usize,
}

#[derive(Debug)]
struct NoneComponent;

impl EcsComponent for NoneComponent {
    const INDEX: u16 = 0;
    const NAME: &'static str = concat!("::", module_path!(), "::NullType");
}

impl EntityComponentIndex {
    pub(crate) fn new(initial_capacity: usize, initial_component_types_width: usize) -> Self {
        EntityComponentIndex {
            component_count: boxed_slice(0, initial_capacity),
            component_types: boxed_slice(
                NoneComponent::get_component_type(),
                initial_capacity * initial_component_types_width,
            ),
            component_types_width: initial_component_types_width,
        }
    }

    pub(crate) fn add_component_type(
        &mut self,
        entity: EntityIndex,
        component_type: ComponentType,
    ) {
        let component_count_before =
            *self.component_count.get(entity.index as usize).unwrap() as usize;
        if component_count_before >= self.component_types_width {
            let mut new_table =
                vec![NoneComponent::get_component_type(); self.component_types.len() * 2]
                    .into_boxed_slice();
            let row_size_before = self.component_types_width;
            self.component_types_width *= 2;
            let row_size_after = self.component_types_width;

            for i in 0..self.component_count.len() {
                for j in 0..row_size_before {
                    new_table[j + i * row_size_after] =
                        self.component_types[i * row_size_before + j];
                }
            }
            self.component_types = new_table;
        }

        let offset = self.component_count.get_mut(entity.index as usize).unwrap();
        *self
            .component_types
            .get_mut(entity.index as usize * self.component_types_width + *offset as usize)
            .unwrap() = component_type;
        *offset += 1;
    }

    pub(crate) fn delete_component_type(
        &mut self,
        entity: EntityIndex,
        component_type: ComponentType,
    ) {
        let entity_index = entity.index as usize;
        for i in 0..self.component_count[entity_index] as usize {
            let this_component_type =
                self.component_types[entity_index * self.component_types_width + i];
            if this_component_type != component_type {
                continue;
            }

            self.component_count[entity_index] -= 1;
            self.component_types[entity_index * self.component_types_width + i] = self
                .component_types[entity_index * self.component_types_width
                + self.component_count[entity_index] as usize];
        }
    }

    pub(crate) fn get_component_types(
        &self,
        entity: EntityIndex,
    ) -> impl Iterator<Item = ComponentType> + '_ {
        let entity = entity.index as usize;

        let copied = self
            .component_types
            .iter()
            .skip(entity * self.component_types_width)
            .take(self.component_count[entity] as usize)
            .copied()
            .collect::<Vec<_>>();
        copied.into_iter()
    }

    pub(crate) fn add_entity(&mut self, entity: EntityIndex) {
        let entity = entity.index as usize;
        if entity >= self.component_count.len() {
            expand_slice(&mut self.component_count, Default::default());
            expand_slice(
                &mut self.component_types,
                NoneComponent::get_component_type(),
            );
        }
        self.component_count[entity] = 0;
    }
}

fn expand_slice<T: Clone>(slice: &mut Box<[T]>, default: T) {
    let mut dst = vec![default; slice.len() * 2].into_boxed_slice();
    dst[0..slice.len()].clone_from_slice(slice);
    *slice = dst;
}

#[cfg(test)]
mod tests {
    use crate::component::ComponentType;
    use crate::internal::entity_component_index::EntityComponentIndex;
    use crate::internal::entity_storage::EntityStorage;

    #[test]
    fn three_components_added_and_read() {
        let mut entities = EntityStorage::with_capacity(512);
        let mut components = EntityComponentIndex::new(512, 8);
        let e1 = entities.new_entity();
        components.add_entity(e1.index);
        components.add_component_type(e1.index, ComponentType { index: 11 });
        components.add_component_type(e1.index, ComponentType { index: 12 });
        components.add_component_type(e1.index, ComponentType { index: 13 });

        let component_types = components.get_component_types(e1.index).collect::<Vec<_>>();
        assert_eq!(
            component_types,
            vec![
                ComponentType { index: 11 },
                ComponentType { index: 12 },
                ComponentType { index: 13 }
            ]
        )
    }

    #[test]
    fn three_components_added_and_read_on_the_second_entity() {
        let mut entities = EntityStorage::with_capacity(512);
        let mut components = EntityComponentIndex::new(512, 8);
        entities.new_entity();
        let e1 = entities.new_entity();
        components.add_entity(e1.index);
        components.add_component_type(e1.index, ComponentType { index: 11 });
        components.add_component_type(e1.index, ComponentType { index: 12 });
        components.add_component_type(e1.index, ComponentType { index: 13 });

        let component_types = components.get_component_types(e1.index).collect::<Vec<_>>();
        assert_eq!(
            component_types,
            vec![
                ComponentType { index: 11 },
                ComponentType { index: 12 },
                ComponentType { index: 13 }
            ]
        )
    }
}
