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

#[test]
fn AppearEventAvailableAfterComponentCreation() {
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
fn AppearEventNotAvailableBeforeCommit() {
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
fn EntityDisappearAvailableAfterComponentRemoval() {
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
fn EntityDisappearDoesntInvokeTwice() {
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
fn EntityDisappearTwice() {
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
fn EntityDisappearNotAvailableBeforeCommit() {
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
fn EntityDisappearNotAvailableBeforeRemoval() {
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
fn EmptyFilterEntityAppearAfterEntityCreated() {
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
fn EmptyFilterEntityAppearNotAvailableAfterEntityCreatedNotCommitted() {
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
fn ABAppearEventForAB() {
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
