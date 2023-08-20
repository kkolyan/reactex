#![allow(non_snake_case)]

use reactex_core::filter::filter_desc::ecs_filter;
use reactex_core::world_mod::world::{ConfigurableWorld, register_query};
use reactex_macro::EcsComponent;

#[derive(EcsComponent, Debug, Default)]
struct A {}

#[derive(EcsComponent, Debug, Default)]
struct B {}

#[test]
fn CommittedEntityQueriedByPreCreatedQuery() {
    let query_A = ecs_filter!(A);
    register_query(query_A);

    let mut world = ConfigurableWorld::new().seal();

    world.query(query_A, |_| {});

    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_A, |e| matched.push(e));

    assert_eq!(matched, vec![e1]);
}

#[test]
fn CommittedEntityQueriedByLateQuery() {
    let query_A = ecs_filter!(A);

    register_query(query_A);
    let mut world = ConfigurableWorld::new().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_A, |e| matched.push(e));

    assert_eq!(matched, vec![e1]);
}

#[test]
fn UnCommittedEntityNotShown() {
    let query_A = ecs_filter!(A);
    register_query(query_A);
    let mut world = ConfigurableWorld::new().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();

    let mut matched = vec![];
    world.query(query_A, |e| matched.push(e));

    assert_eq!(matched, vec![]);
}

#[test]
fn ANotMatchesB() {
    let query_B = ecs_filter!(B);
    register_query(query_B);
    let mut world = ConfigurableWorld::new().seal();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    let eB = world.create_entity();
    world.add_component(eB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_B, |e| matched.push(e));

    assert_eq!(matched, vec![eB]);
}

#[test]
fn EmptyNotMatches() {
    let query_B = ecs_filter!(B);
    register_query(query_B);
    let mut world = ConfigurableWorld::new().seal();
    world.create_entity();
    let eB = world.create_entity();
    world.add_component(eB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_B, |e| matched.push(e));

    assert_eq!(matched, vec![eB]);
}

#[test]
fn ABMatchesAB() {
    let query_AB = ecs_filter!(A, B);
    register_query(query_AB);
    let mut world = ConfigurableWorld::new().seal();
    let eAB = world.create_entity();
    world.add_component(eAB, A::default()).unwrap();
    world.add_component(eAB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_AB, |e| matched.push(e));

    assert_eq!(matched, vec![eAB]);
}

#[test]
fn ANotMatchedToAB() {
    let query_AB = ecs_filter!(A, B);
    register_query(query_AB);
    let mut world = ConfigurableWorld::new().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_AB, |e| matched.push(e));

    assert_eq!(matched, vec![]);
}

#[test]
fn ABMatchedToA() {
    let query_A = ecs_filter!(A);
    register_query(query_A);
    let mut world = ConfigurableWorld::new().seal();
    let eAB = world.create_entity();
    world.add_component(eAB, A::default()).unwrap();
    world.add_component(eAB, B::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_A, |e| matched.push(e));

    assert_eq!(matched, vec![eAB]);
}

#[test]
fn EmptyMatchesEmpty() {
    let query_all = ecs_filter!();
    register_query(query_all);
    let mut world = ConfigurableWorld::new().seal();
    let eEmpty = world.create_entity();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_all, |e| matched.push(e));

    assert_eq!(matched, vec![eEmpty]);
}

#[test]
fn AMatchesEmpty() {
    let query_all = ecs_filter!();
    register_query(query_all);
    let mut world = ConfigurableWorld::new().seal();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    world.execute_all();

    let mut matched = vec![];
    world.query(query_all, |e| matched.push(e));

    assert_eq!(matched, vec![eA]);
}

/*

*/
