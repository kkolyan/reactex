#![allow(non_snake_case)]

use reactex_core::ecs_filter;
use reactex_core::world::World;
use reactex_macro::EcsComponent;

#[derive(EcsComponent, Debug, Default)]
struct A {}

#[derive(EcsComponent, Debug, Default)]
struct B {}

#[test]
fn CommittedEntityQueriedByPreCreatedQuery() {
    let mut world = World::new();

    world.query(ecs_filter!(A), |_| {});

    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A), |e| matched.push(e));

    assert_eq!(matched, vec![e1]);
}

#[test]
fn CommittedEntityQueriedByLateQuery() {
    let mut world = World::new();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A), |e| matched.push(e));

    assert_eq!(matched, vec![e1]);
}

#[test]
fn UnCommittedEntityNotShown() {
    let mut world = World::new();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();

    let mut matched = vec![];
    world.query(ecs_filter!(A), |e| matched.push(e));

    assert_eq!(matched, vec![]);
}

#[test]
fn ANotMatchesB() {
    let mut world = World::new();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    let eB = world.create_entity();
    world.add_component(eB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(B), |e| matched.push(e));

    assert_eq!(matched, vec![eB]);
}

#[test]
fn EmptyNotMatches() {
    let mut world = World::new();
    world.create_entity();
    let eB = world.create_entity();
    world.add_component(eB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(B), |e| matched.push(e));

    assert_eq!(matched, vec![eB]);
}

#[test]
fn ABMatchesAB() {
    let mut world = World::new();
    let eAB = world.create_entity();
    world.add_component(eAB, A::default()).unwrap();
    world.add_component(eAB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A, B), |e| matched.push(e));

    assert_eq!(matched, vec![eAB]);
}

#[test]
fn ANotMatchedToAB() {
    let mut world = World::new();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A, B), |e| matched.push(e));

    assert_eq!(matched, vec![]);
}

#[test]
fn ABMatchedToA() {
    let mut world = World::new();
    let eAB = world.create_entity();
    world.add_component(eAB, A::default()).unwrap();
    world.add_component(eAB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A), |e| matched.push(e));

    assert_eq!(matched, vec![eAB]);
}

#[test]
fn EmptyMatchesEmpty() {
    let mut world = World::new();
    let eEmpty = world.create_entity();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(), |e| matched.push(e));

    assert_eq!(matched, vec![eEmpty]);
}

#[test]
fn AMatchesEmpty() {
    let mut world = World::new();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(), |e| matched.push(e));

    assert_eq!(matched, vec![eA]);
}

/*

*/
