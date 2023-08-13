#![allow(non_snake_case)]

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use reactex_core::ecs_filter;
use reactex_core::world::World;
use reactex_macro::EcsComponent;

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct A
{}

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct B
{}

#[test]
fn AppearEventAvailableAfterComponentCreation()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddAppearHandler("test", ecs_filter!(A), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn AppearEventNotAvailableBeforeCommit()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddAppearHandler("test", ecs_filter!(A), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    assert_eq!(matched.borrow().deref(), &vec! {});
}

#[test]
fn EntityDisappearAvailableAfterComponentRemoval()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddDisappearHandler("test", ecs_filter!(A), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.borrow().deref(), &vec!());
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn EntityDisappearNotAvailableBeforeCommit()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddDisappearHandler("test", ecs_filter!(A), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.borrow().deref(), &vec!());
}

#[test]
fn EntityDisappearNotAvailableBeforeRemoval()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddDisappearHandler("test", ecs_filter!(A), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec!());
}

#[test]
fn EmptyFilterEntityAppearAfterEntityCreated()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddAppearHandler("test", ecs_filter!(), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}

#[test]
fn EmptyFilterEntityAppearNotAvailableAfterEntityCreatedNotCommitted()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddDisappearHandler("test", ecs_filter!(), move |entity| matched.borrow_mut().push(entity));
    }
    let _eA = world.create_entity();
    assert_eq!(matched.borrow().deref(), &vec!());
}

#[test]
fn ABAppearEventForAB()
{
    let matched = Rc::new(RefCell::new(Vec::new()));
    let mut world = World::new();
    {
        let matched = matched.clone();
        world.AddAppearHandler("test", ecs_filter!(A, B), move |entity| matched.borrow_mut().push(entity));
    }
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.add_component(eA, B {}).unwrap();
    world.execute_all();
    assert_eq!(matched.borrow().deref(), &vec! {eA});
}