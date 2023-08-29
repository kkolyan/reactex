#![allow(non_snake_case)]

use reactex_core::ecs_filter;
use reactex_core::ConfigurableWorld;
use reactex_core::World;
use reactex_macro::EcsComponent;
use to_vec::ToVec;

#[derive(EcsComponent, Debug, Default)]
struct A {}

#[derive(EcsComponent, Debug, Default)]
struct B {}

#[test]
fn CommittedEntityQueriedByPreCreatedQuery() {
    let query_A = ecs_filter!(A);
    World::register_query(query_A);

    let mut world = ConfigurableWorld::create_for_test().seal();

    let _ = world.query(query_A);

    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_A).to_vec();

    assert_eq!(matched, vec![e1]);
}

#[test]
fn CommittedEntityQueriedByLateQuery() {
    let query_A = ecs_filter!(A);

    World::register_query(query_A);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_A).to_vec();

    assert_eq!(matched, vec![e1]);
}

#[test]
fn UnCommittedEntityNotShown() {
    let query_A = ecs_filter!(A);
    World::register_query(query_A);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();

    let matched = world.query(query_A).to_vec();

    assert_eq!(matched, vec![]);
}

#[test]
fn ANotMatchesB() {
    let query_B = ecs_filter!(B);
    World::register_query(query_B);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    let eB = world.create_entity();
    world.add_component(eB, B::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_B).to_vec();

    assert_eq!(matched, vec![eB]);
}

#[test]
fn EmptyNotMatches() {
    let query_B = ecs_filter!(B);
    World::register_query(query_B);
    let mut world = ConfigurableWorld::create_for_test().seal();
    world.create_entity();
    let eB = world.create_entity();
    world.add_component(eB, B::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_B).to_vec();

    assert_eq!(matched, vec![eB]);
}

#[test]
fn ABMatchesAB() {
    let query_AB = ecs_filter!(A, B);
    World::register_query(query_AB);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let eAB = world.create_entity();
    world.add_component(eAB, A::default()).unwrap();
    world.add_component(eAB, B::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_AB).to_vec();

    assert_eq!(matched, vec![eAB]);
}

#[test]
fn ANotMatchedToAB() {
    let query_AB = ecs_filter!(A, B);
    World::register_query(query_AB);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let e1 = world.create_entity();
    world.add_component(e1, A::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_AB).to_vec();

    assert_eq!(matched, vec![]);
}

#[test]
fn ABMatchedToA() {
    let query_A = ecs_filter!(A);
    World::register_query(query_A);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let eAB = world.create_entity();
    world.add_component(eAB, A::default()).unwrap();
    world.add_component(eAB, B::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_A).to_vec();

    assert_eq!(matched, vec![eAB]);
}

#[test]
fn EmptyMatchesEmpty() {
    let query_all = ecs_filter!();
    World::register_query(query_all);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let eEmpty = world.create_entity();
    world.execute_all();

    let matched = world.query(query_all).to_vec();

    assert_eq!(matched, vec![eEmpty]);
}

#[test]
fn AMatchesEmpty() {
    let query_all = ecs_filter!();
    World::register_query(query_all);
    let mut world = ConfigurableWorld::create_for_test().seal();
    let eA = world.create_entity();
    world.add_component(eA, A::default()).unwrap();
    world.execute_all();

    let matched = world.query(query_all).to_vec();

    assert_eq!(matched, vec![eA]);
}

/*

*/
