#![allow(non_snake_case)]

use reactex_core::ecs_filter;
use reactex_core::ConfigurableWorld;
use reactex_macro::EcsComponent;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

#[derive(EcsComponent, Debug, Eq, PartialEq)]
struct A {}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
struct Signal {
    Value: i32,
}

impl Signal {
    fn new(Value: i32) -> Signal {
        Signal { Value }
    }
}

#[derive(Debug, Copy, Clone)]
struct AnotherSignal();

#[test]
fn GlobalSignalReceived() {
    let received = Rc::new(RefCell::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.borrow_mut().push(*ctx.signal)
        });
    }
    let mut world = world.seal();
    world.signal(Signal::new(42));
    world.execute_all();

    assert_eq!(received.borrow().deref(), &vec! {Signal::new(42)});
}

#[test]
fn GlobalSignalNotReceivedBeforeExecution() {
    let received = Rc::new(RefCell::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.borrow_mut().push(*ctx.signal)
        });
    }
    let mut world = world.seal();
    world.signal(Signal::new(Default::default()));

    assert_eq!(received.borrow().deref(), &vec! {});
}

#[test]
fn GlobalSignalReceivedInOrderOfSubmission() {
    let received = Rc::new(RefCell::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.borrow_mut().push(*ctx.signal)
        })
    };
    let mut world = world.seal();
    world.signal(Signal::new(17));
    world.signal(Signal::new(42));
    world.execute_all();

    assert_eq!(
        received.borrow().deref(),
        &vec! {Signal::new(17), Signal::new(42)}
    );
}

#[test]
fn GlobalSignalReceivedInOrderOfSubmissionDifferentTypes() {
    let received = Rc::new(RefCell::new(Vec::<Box<dyn Debug>>::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<AnotherSignal>("test", move |ctx| {
            received.borrow_mut().push(Box::new(*ctx.signal))
        });
    }
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.borrow_mut().push(Box::new(*ctx.signal))
        });
    }
    let mut world = world.seal();
    world.signal(Signal::new(17));
    world.signal(AnotherSignal());
    world.execute_all();

    assert_eq!(
        format!("{:?}", received.borrow().deref()),
        format!(
            "{:?}",
            vec! {Box::new(Signal::new(17)) as Box<dyn Debug>, Box::new(AnotherSignal())}
        )
    );
}

#[test]
fn GlobalSignalReceivedTransitiveAfterExecuteAll() {
    let received = Rc::new(RefCell::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        world.add_global_signal_handler::<AnotherSignal>("test", move |ctx| {
            ctx.send_signal(Signal::new(17))
        });
    }
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.borrow_mut().push(*ctx.signal)
        });
    }
    let mut world = world.seal();

    world.signal(AnotherSignal());
    world.execute_all();

    assert_eq!(received.borrow().deref(), &vec! {Signal::new(17)});
}

#[test]
fn EntityMatchedAndSignalReceived() {
    let received_signals = Rc::new(RefCell::new(vec![]));
    let matched_entities = Rc::new(RefCell::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received_signals = received_signals.clone();
        let matched_entities = matched_entities.clone();
        world.add_entity_signal_handler::<Signal>("test", ecs_filter!(A), move |ctx, entity| {
            received_signals.borrow_mut().push(*ctx.signal);
            matched_entities.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();

    let e1 = world.create_entity();
    world.add_component(e1, A {}).unwrap();
    world.execute_all();

    world.signal(Signal::new(17));
    world.execute_all();

    assert_eq!(received_signals.borrow().deref(), &vec! {Signal::new(17)});
    assert_eq!(matched_entities.borrow().deref(), &vec! {e1});
}

#[test]
fn NotEntityMatchedAndSignalReceived() {
    let received_signals = Rc::new(RefCell::new(vec![]));
    let matched_entities = Rc::new(RefCell::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received_signals = received_signals.clone();
        let matched_entities = matched_entities.clone();
        world.add_entity_signal_handler::<Signal>("test", ecs_filter!(A), move |ctx, entity| {
            received_signals.borrow_mut().push(*ctx.signal);
            matched_entities.borrow_mut().push(entity)
        });
    }
    let mut world = world.seal();

    let _e1 = world.create_entity();

    world.signal(Signal::new(17));
    world.execute_all();

    assert_eq!(received_signals.borrow().deref(), &vec! {});
    assert_eq!(matched_entities.borrow().deref(), &vec! {});
}
