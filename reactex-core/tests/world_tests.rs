use reactex_core::ecs_component;
use reactex_core::entity::EntityKey;
use reactex_core::lang::ref_map_result_option;
use reactex_core::world::EntityError;
use reactex_core::world::World;
use reactex_core::world::WorldError;
use reactex_core::world::WorldResult;
use reactex_core::StaticComponentType;
use std::cell::Ref;
use std::cell::RefCell;
use std::fmt::Debug;
use std::fs;
use std::rc::Rc;
use ctor::ctor;
use log::info;

#[derive(Default, Debug)]
struct A {
    value: i32,
}

ecs_component! {A}

struct Entity {
    key: EntityKey,
    world: Rc<RefCell<World>>,
}

impl Entity {
    pub(crate) fn destroy_entity(&self) -> WorldResult {
        self.world.borrow_mut().destroy_entity(self.key)
    }
}

impl Entity {}

impl Entity {
    fn exists(&self) -> bool {
        self.world.borrow_mut().entity_exists(self.key)
    }

    fn has<T: StaticComponentType>(&self) -> WorldResult<bool> {
        self.world.borrow().has_component::<T>(self.key)
    }
    fn add<T: StaticComponentType>(&self, value: T) -> WorldResult {
        self.world.borrow_mut().add_component::<T>(self.key, value)
    }
    fn get<T: StaticComponentType>(&self) -> WorldResult<Option<Ref<T>>> {
        ref_map_result_option(self.world.borrow(), |it| it.get_component::<T>(self.key))
    }
    fn remove<T: StaticComponentType>(&self) -> WorldResult {
        self.world.borrow_mut().remove_component::<T>(self.key)
    }

    fn modify<T: StaticComponentType>(&self, action: impl FnOnce(&mut T) + 'static) -> WorldResult {
        self.world
            .borrow_mut()
            .modify_component(self.key, Box::new(action))
    }
}

trait TestWorld {
    fn new_entity(&self) -> Entity;
    fn execute_all(&self);
}

impl TestWorld for Rc<RefCell<World>> {
    fn new_entity(&self) -> Entity {
        Entity {
            key: self.borrow_mut().create_entity(),
            world: self.clone(),
        }
    }

    fn execute_all(&self) {
        self.borrow_mut().execute_all();
    }
}

#[ctor]
fn init_logging() {
    log4rs::init_file("tests/log4rs.test.yaml", Default::default()).unwrap();
    println!("test started");
}

fn create_world() -> Rc<RefCell<World>> {
    Rc::new(RefCell::new(World::new()))
}

#[test]
fn entity_exists_immediately() {
    let world = create_world();
    assert!(world.new_entity().exists());
}

#[test]
fn entity_could_be_deleted_immediately() {
    let world = create_world();
    let entity = world.new_entity();
    entity.destroy_entity().unwrap();
    assert!(!entity.exists());
}

#[test]
fn component_can_be_added_to_uncommitted_entity() {
    let world = create_world();
    world.new_entity().add(A::default()).unwrap();
}

#[test]
fn component_cannot_be_deleted_from_uncommitted_entity() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    assert_eq!(
        entity.remove::<A>(),
        Err(WorldError::Entity(EntityError::NotCommitted))
    )
}

#[test]
fn component_cannot_be_checked_on_uncommitted_entity() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    assert_eq!(
        entity.has::<A>(),
        Err(WorldError::Entity(EntityError::NotCommitted))
    );
}

#[test]
fn component_cannot_be_read_on_uncommitted_entity() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    assert_eq!(
        entity.get::<A>().unwrap_err(),
        WorldError::Entity(EntityError::NotCommitted)
    )
}

#[test]
fn component_checked_after_commit() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();

    assert!(entity.has::<A>().unwrap());
}

#[test]
fn component_state_available_after_commit() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A { value: 42 }).unwrap();
    world.execute_all();

    assert_eq!(entity.get::<A>().unwrap().unwrap().value, 42);
}

#[test]
fn component_state_change_visible_between_reads() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();

    entity.modify::<A>(|it| it.value = 42).unwrap();

    assert_eq!(entity.get::<A>().unwrap().unwrap().value, 42);
}

#[test]
fn component_delete_uncommitted_not_visible() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();
    entity.remove::<A>().unwrap();
    assert!(entity.has::<A>().unwrap());
}

#[test]
fn component_delete_visible_after_commit() {
    let world = create_world();
    let entity = world.new_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();
    entity.remove::<A>().unwrap();
    world.execute_all();
    assert!(!entity.has::<A>().unwrap());
}

#[test]
fn component_add_delete_visible_after_commit() {
    let world = create_world();
    let entity = world.new_entity();
    world.execute_all();
    entity.add(A::default()).unwrap();
    entity.remove::<A>().unwrap();
    world.execute_all();
    assert!(!entity.has::<A>().unwrap());
}
