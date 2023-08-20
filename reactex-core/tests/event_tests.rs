#![allow(non_snake_case)]

use reactex_core::filter::filter_desc::ecs_filter;
use reactex_core::world_mod::world::ConfigurableWorld;
use reactex_macro::EcsComponent;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct A {}

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct B {}

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct C {
    value: i32,
}

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct D {
    value: i32,
}

#[test]
fn appear_event_available_after_component_creation() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        })
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn appear_event_not_available_before_commit() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    assert_eq!(matched.borrow().deref(), &vec! {});
}

#[test]
fn entity_disappear_available_after_component_removal() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.borrow().deref(), &vec!());
    world.execute_all();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn entity_disappear_doesnt_invoke_twice() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.borrow().deref(), &vec!());
    world.execute_all();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn entity_disappear_twice() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();

    {
        let eA = world.create_entity();
        world.add_component(eA, A {}).unwrap();
        world.execute_all();
        world.remove_component::<A>(eA).unwrap();
        world.execute_all();

        matched.borrow_mut().clear();
    }

    {
        let eA = world.create_entity();
        world.add_component(eA, A {}).unwrap();
        world.execute_all();
        world.remove_component::<A>(eA).unwrap();
        world.execute_all();

        assert_eq!(matched.borrow().deref(), &vec! {eA});
    }
}

#[test]
fn entity_disappear_not_available_before_commit() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.borrow().deref(), &vec!());
}

#[test]
fn entity_disappear_not_available_before_removal() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec!());
}

#[test]
fn empty_filter_entity_appear_after_entity_created() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn empty_filter_entity_disappear_after_entity_destroyed() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.execute_all();
    world.destroy_entity(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn empty_filter_entity_appear_not_available_after_entity_created_not_committed() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let _eA = world.create_entity();
    assert_eq!(matched.borrow().deref(), &vec!());
}

#[test]
fn abappear_event_for_ab() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(A, B), move |entity, _, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.add_component(eA, B {}).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn components_available_during_disappear_event() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(C, D), move |entity, stable, _| {
            assert_eq!(stable.get_component::<C>(entity).unwrap().unwrap().value, 7);
            assert_eq!(stable.get_component::<D>(entity).unwrap().unwrap().value, 8);
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, C { value: 7 }).unwrap();
    world.add_component(eA, D { value: 8 }).unwrap();
    world.execute_all();
    world.remove_component::<C>(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn removed_component_doesnt_fire_similar_disappears() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A, B), move |entity, stable, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {});
}

#[test]
fn destroyed_entity_doesnt_fire_similar_disappears() {
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = ConfigurableWorld::new();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A, B), move |entity, stable, _| {
            matched.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.destroy_entity(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {});
}
