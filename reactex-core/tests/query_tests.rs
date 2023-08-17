#![allow(non_snake_case)]

use reactex_core::filter::filter_desc::ecs_filter;
use reactex_core::world_mod::world::ConfigurableWorld;
use reactex_macro::EcsComponent;

#[derive(EcsComponent, Debug, Default)]
struct A {}

#[derive(EcsComponent, Debug, Default)]
struct B {}

#[test]
fn CommittedEntityQueriedByPreCreatedQuery() {
    let mut world = ConfigurableWorld::new().seal();

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
    let mut world = ConfigurableWorld::new().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A), |e| matched.push(e));

    assert_eq!(matched, vec![e1]);
}

#[test]
fn UnCommittedEntityNotShown() {
    let mut world = ConfigurableWorld::new().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();

    let mut matched = vec![];
    world.query(ecs_filter!(A), |e| matched.push(e));

    assert_eq!(matched, vec![]);
}

#[test]
fn ANotMatchesB() {
    let mut world = ConfigurableWorld::new().seal();
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
    let mut world = ConfigurableWorld::new().seal();
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
    let mut world = ConfigurableWorld::new().seal();
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
    let mut world = ConfigurableWorld::new().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(A, B), |e| matched.push(e));

    assert_eq!(matched, vec![]);
}

#[test]
fn ABMatchedToA() {
    let mut world = ConfigurableWorld::new().seal();
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
    let mut world = ConfigurableWorld::new().seal();
    let eEmpty = world.create_entity();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(), |e| matched.push(e));

    assert_eq!(matched, vec![eEmpty]);
}

#[test]
fn AMatchesEmpty() {
    let mut world = ConfigurableWorld::new().seal();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(ecs_filter!(), |e| matched.push(e));

    assert_eq!(matched, vec![eA]);
}

/*

*/
