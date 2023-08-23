#![allow(non_snake_case)]

use reactex_core::ecs_filter;
use reactex_core::ConfigurableWorld;
use reactex_macro::EcsComponent;

use std::ops::Deref;
use std::rc::Rc;
use std::sync::Mutex;

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
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        })
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn appear_event_not_available_before_commit() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {});
}

#[test]
fn entity_disappear_available_after_component_removal() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.lock().unwrap().deref(), &vec!());
    world.execute_all();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn entity_disappear_doesnt_invoke_twice() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.lock().unwrap().deref(), &vec!());
    world.execute_all();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn entity_disappear_twice() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();

    {
        let eA = world.create_entity();
        world.add_component(eA, A {}).unwrap();
        world.execute_all();
        world.remove_component::<A>(eA).unwrap();
        world.execute_all();

        matched.lock().unwrap().clear();
    }

    {
        let eA = world.create_entity();
        world.add_component(eA, A {}).unwrap();
        world.execute_all();
        world.remove_component::<A>(eA).unwrap();
        world.execute_all();

        assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
    }
}

#[test]
fn entity_disappear_not_available_before_commit() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    assert_eq!(matched.lock().unwrap().deref(), &vec!());
}

#[test]
fn entity_disappear_not_available_before_removal() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec!());
}

#[test]
fn empty_filter_entity_appear_after_entity_created() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn empty_filter_entity_disappear_after_entity_destroyed() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.execute_all();
    world.destroy_entity(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn empty_filter_entity_appear_not_available_after_entity_created_not_committed() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let _eA = world.create_entity();
    assert_eq!(matched.lock().unwrap().deref(), &vec!());
}

#[test]
fn abappear_event_for_ab() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_appear_handler("test", ecs_filter!(A, B), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.add_component(eA, B {}).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn components_available_during_disappear_event() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(C, D), move |ctx, entity| {
            assert_eq!(ctx.get_entity(entity).unwrap().get::<C>().unwrap().value, 7);
            assert_eq!(ctx.get_entity(entity).unwrap().get::<D>().unwrap().value, 8);
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, C { value: 7 }).unwrap();
    world.add_component(eA, D { value: 8 }).unwrap();
    world.execute_all();
    world.remove_component::<C>(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {eA});
}

#[test]
fn removed_component_doesnt_fire_similar_disappears() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A, B), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.remove_component::<A>(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {});
}

#[test]
fn destroyed_entity_doesnt_fire_similar_disappears() {
    let matched = Rc::new(Mutex::new(Vec::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let matched = matched.clone();
        world.add_disappear_handler("test", ecs_filter!(A, B), move |_, entity| {
            matched.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();
    let eA = world.create_entity();
    world.add_component(eA, A {}).unwrap();
    world.execute_all();
    world.destroy_entity(eA).unwrap();
    world.execute_all();
    assert_eq!(matched.lock().unwrap().deref(), &vec! {});
}
