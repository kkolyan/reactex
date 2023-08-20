use std::fmt::Debug;

use ctor::ctor;
use reactex_core::{ConfigurableWorld, EcsContainer, EcsContainerBuilder};
use reactex_core::ComponentError;
use reactex_core::EntityError;
use reactex_core::World;
use reactex_core::WorldError;
use reactex_macro::EcsComponent;

#[derive(Default, Debug, EcsComponent)]
struct A {
    value: i32,
}

#[derive(Default, Debug, EcsComponent)]
struct W {
    a: i32,
    b: i32,
}

#[derive(Debug, EcsComponent)]
struct X {
    value: i32,
}

#[derive(Debug, EcsComponent)]
struct Y {
    value: i32,
}

struct NotCopy<T> {
    value: T,
}

#[ctor]
fn init_logging() {
    log4rs::init_file("tests/log4rs.test.yaml", Default::default()).unwrap();
    println!("test started");
}

fn create_world() -> EcsContainer {
    
    let mut world = EcsContainer::create().seal();

    // just noise to avoid false positives due to zero indexes or absence of interference

    world.execute_once(|ctx| {
        {
            let entity = ctx.create_entity();
            entity.add(X { value: 0 });
        }
        {
            let entity = ctx.create_entity();
            entity.add(Y { value: 0 });
        }
        {
            let entity = ctx.create_entity();
            entity.add(X { value: 0 });
            entity.add(Y { value: 0 });
        }
        ctx.create_entity();
    });
    world
}

#[test]
fn entity_could_be_deleted_immediately() {
    let mut world = create_world();
    let mut created = None;
    world.execute_once(|ctx| {
        let entity = ctx.create_entity();
        created = Some(entity.key());
        entity.destroy();
    });

    world.execute_once(|ctx| {
        assert!(!ctx.get_entity(created.unwrap()).is_none());
    });
}

#[test]
fn entity_could_be_deleted_immediately_so_cannot_add_component() {
    let mut world = create_world();
    world.execute_once(|ctx| {
        let entity = ctx.create_entity();
        entity.clone().destroy();
        let result = entity.try_add(A { value: 17 });
        assert_eq!(result, Err(WorldError::Entity(EntityError::NotExists)));
    });
}

#[test]
fn component_can_be_added_to_uncommitted_entity() {
    let mut world = create_world();
    world.execute_once(|world| {
        let entity = world.create_entity();
        entity.add(A::default());
    });
}

#[test]
fn uncommitted_component_can_be_deleted() {
    let mut world = create_world();
    let mut entity = None;
    world.execute_once(|world| {
        entity = Some(world.create_entity().key());
    });
    world.execute_once(|world| {
        let entity = world.get_entity(entity.unwrap()).unwrap();
        entity.add(A::default()).unwrap();
        assert_eq!(world.remove_component::<A>(entity), Ok(()))
    });
}

#[test]
fn non_existent_component_cannot_be_deleted() {
    let mut world = create_world();
    let entity = world.create_entity();
    world.execute_all();
    assert_eq!(
        world.remove_component::<A>(entity),
        Err(WorldError::Component(ComponentError::NotFound))
    )
}

#[test]
fn component_cannot_be_checked_on_uncommitted_entity() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    assert_eq!(
        world.has_component::<A>(entity),
        Err(WorldError::Entity(EntityError::NotCommitted))
    );
}

#[test]
fn component_cannot_be_read_on_uncommitted_entity() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    assert_eq!(
        world.get_component::<A>(entity).unwrap_err(),
        WorldError::Entity(EntityError::NotCommitted)
    )
}

#[test]
fn component_checked_after_commit() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();

    assert!(world.has_component::<A>(entity).unwrap());
}

#[test]
fn component_state_available_after_commit() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A { value: 42 }).unwrap();
    world.execute_all();

    assert_eq!(world.get_component::<A>(entity).unwrap().unwrap().value, 42);
}

#[test]
fn component_state_change_not_visible_before_commit() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A { value: 17 }).unwrap();
    world.execute_all();

    world
        .modify_component::<A>(entity, |it| it.value = 42)
        .unwrap();

    assert_eq!(world.get_component::<A>(entity).unwrap().unwrap().value, 17);
}

#[test]
fn component_state_change_visible_after_commit() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();

    world
        .modify_component::<A>(entity, |it| it.value = 42)
        .unwrap();
    world.execute_all();

    assert_eq!(world.get_component::<A>(entity).unwrap().unwrap().value, 42);
}

#[test]
fn component_state_change_may_use_closure() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();

    let value = NotCopy { value: 42 };

    world
        .modify_component::<A>(entity, move |it| it.value = value.value)
        .unwrap();
    println!("value: {}", value.value);
    world.execute_all();

    assert_eq!(world.get_component::<A>(entity).unwrap().unwrap().value, 42);
}

#[test]
fn component_state_changes_merged() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(W { a: 17, b: 42 }).unwrap();
    world.execute_all();

    world.modify_component::<W>(entity, |it| it.a += 1).unwrap();
    world.modify_component::<W>(entity, |it| it.b += 3).unwrap();
    world.execute_all();

    let x = world.get_component::<W>(entity).unwrap().unwrap();

    assert_eq!((x.a, x.b), (18, 45));
}

#[test]
fn component_delete_uncommitted_not_visible() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();
    world.remove_component::<A>(entity).unwrap();
    assert!(world.has_component::<A>(entity).unwrap());
}

#[test]
fn component_delete_visible_after_commit() {
    let mut world = create_world();
    let entity = world.create_entity();
    entity.add(A::default()).unwrap();
    world.execute_all();
    world.remove_component::<A>(entity).unwrap();
    world.execute_all();
    assert!(!world.has_component::<A>(entity).unwrap());
}

#[test]
fn component_add_delete_visible_after_commit() {
    let mut world = create_world();
    let entity = world.create_entity();
    world.execute_all();
    entity.add(A::default()).unwrap();
    world.remove_component::<A>(entity).unwrap();
    world.execute_all();
    assert!(!world.has_component::<A>(entity).unwrap());
}
