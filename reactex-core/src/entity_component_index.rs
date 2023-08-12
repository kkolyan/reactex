use crate::entity::EntityIndex;
use crate::lang::boxed_slice;
use crate::ComponentType;
use crate::StaticComponentType;

pub(crate) struct EntityComponentIndex {
    component_count: Box<[u16]>,
    component_types: Box<[ComponentType]>,
    component_types_width: usize,
}

#[derive(Debug)]
struct NoType;

impl StaticComponentType for NoType {
    const INDEX: u16 = 0;
    const NAME: &'static str = concat!(module_path!(), "::NoType");
}

impl EntityComponentIndex {
    pub fn new(initial_capacity: usize, initial_component_types_width: usize) -> Self {
        EntityComponentIndex {
            component_count: boxed_slice(0, initial_capacity),
            component_types: boxed_slice(
                NoType::get_component_type(),
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
            let mut new_table = vec![NoType::get_component_type(); self.component_types.len() * 2]
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

        *self.component_count.get_mut(entity.index as usize).unwrap() += 1;
        *self.component_types.get_mut(entity.index as usize).unwrap() = component_type;
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

        self.component_types
            .iter()
            .skip(entity * self.component_types_width)
            .take(self.component_count[entity] as usize)
            .copied()
    }

    pub(crate) fn add_entity(&mut self, entity: EntityIndex) {
        let entity = entity.index as usize;
        if entity >= self.component_count.len() {
            expand_slice(&mut self.component_count, Default::default());
            expand_slice(&mut self.component_types, NoType::get_component_type());
        }
        self.component_count[entity] = 0;
    }
}

fn expand_slice<T: Clone>(slice: &mut Box<[T]>, default: T) {
    let mut dst = vec![default; slice.len() * 2].into_boxed_slice();
    dst[0..slice.len()].clone_from_slice(slice);
    *slice = dst;
}
