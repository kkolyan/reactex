#![allow(non_snake_case)]

use ctor::ctor;
use reactex_core::ecs_filter;
use reactex_core::ConfigurableWorld;
use reactex_core::EcsContainer;
use reactex_macro::EcsComponent;

use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Mutex;

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

#[ctor]
fn init_logging() {
    // let _ = log4rs::init_file("tests/log4rs.test.yaml", Default::default());
    println!("test started");
}

#[test]
fn GlobalSignalReceived() {
    let received = Rc::new(Mutex::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.lock().unwrap().push(*ctx.signal)
        });
    }
    let mut world = world.seal();
    world.signal(Signal::new(42));
    world.execute_all();

    assert_eq!(received.lock().unwrap().deref(), &vec! {Signal::new(42)});
}

#[test]
fn GlobalSignalNotReceivedBeforeExecution() {
    let received = Rc::new(Mutex::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.lock().unwrap().push(*ctx.signal)
        });
    }
    let mut world = world.seal();
    world.signal(Signal::new(Default::default()));

    assert_eq!(received.lock().unwrap().deref(), &vec! {});
}

#[test]
fn GlobalSignalReceivedInOrderOfSubmission() {
    let received = Rc::new(Mutex::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.lock().unwrap().push(*ctx.signal)
        })
    };
    let mut world = world.seal();
    world.signal(Signal::new(17));
    world.signal(Signal::new(42));
    world.execute_all();

    assert_eq!(
        received.lock().unwrap().deref(),
        &vec! {Signal::new(17), Signal::new(42)}
    );
}

#[test]
fn GlobalSignalReceivedInOrderOfSubmissionDifferentTypes() {
    let received = Rc::new(Mutex::new(Vec::<Box<dyn Debug>>::new()));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received = received.clone();
        world.add_global_signal_handler::<AnotherSignal>("test", move |ctx| {
            received.lock().unwrap().push(Box::new(*ctx.signal))
        });
    }
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.lock().unwrap().push(Box::new(*ctx.signal))
        });
    }
    let mut world = world.seal();
    world.signal(Signal::new(17));
    world.signal(AnotherSignal());
    world.execute_all();

    assert_eq!(
        format!("{:?}", received.lock().unwrap().deref()),
        format!(
            "{:?}",
            vec! {Box::new(Signal::new(17)) as Box<dyn Debug>, Box::new(AnotherSignal())}
        )
    );
}

#[test]
fn GlobalSignalReceivedTransitiveAfterExecuteAll() {
    let received = Rc::new(Mutex::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        world.add_global_signal_handler::<AnotherSignal>("test", move |ctx| {
            ctx.send_signal(Signal::new(17))
        });
    }
    {
        let received = received.clone();
        world.add_global_signal_handler::<Signal>("test", move |ctx| {
            received.lock().unwrap().push(*ctx.signal)
        });
    }
    let mut world = world.seal();

    world.signal(AnotherSignal());
    world.execute_all();

    assert_eq!(received.lock().unwrap().deref(), &vec! {Signal::new(17)});
}

#[test]
fn EntityMatchedAndSignalReceived() {
    let received_signals = Rc::new(Mutex::new(vec![]));
    let matched_entities = Rc::new(Mutex::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received_signals = received_signals.clone();
        let matched_entities = matched_entities.clone();
        world.add_entity_signal_handler::<Signal>("test", ecs_filter!(A), move |ctx, entity| {
            received_signals.lock().unwrap().push(*ctx.signal);
            matched_entities.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();

    let e1 = world.create_entity();
    world.add_component(e1, A {}).unwrap();
    world.execute_all();

    world.signal(Signal::new(17));
    world.execute_all();

    assert_eq!(
        received_signals.lock().unwrap().deref(),
        &vec! {Signal::new(17)}
    );
    assert_eq!(matched_entities.lock().unwrap().deref(), &vec! {e1});
}

#[test]
fn NotEntityMatchedAndSignalReceived() {
    let received_signals = Rc::new(Mutex::new(vec![]));
    let matched_entities = Rc::new(Mutex::new(vec![]));
    let mut world = ConfigurableWorld::create_for_test();
    {
        let received_signals = received_signals.clone();
        let matched_entities = matched_entities.clone();
        world.add_entity_signal_handler::<Signal>("test", ecs_filter!(A), move |ctx, entity| {
            received_signals.lock().unwrap().push(*ctx.signal);
            matched_entities.lock().unwrap().push(entity)
        });
    }
    let mut world = world.seal();

    let _e1 = world.create_entity();

    world.signal(Signal::new(17));
    world.execute_all();

    assert_eq!(received_signals.lock().unwrap().deref(), &vec! {});
    assert_eq!(matched_entities.lock().unwrap().deref(), &vec! {});
}

#[test]
fn cancelled_entity_removed_from_filter() {
    let signals = Rc::new(Mutex::new(0));
    let mut ecs = EcsContainer::create()
        .configure_in_test(|world| {
            let signals = signals.clone();
            world.add_entity_signal_handler::<Signal>("test", ecs_filter!(A), move |_, _| {
                *signals.lock().unwrap() += 1;
            })
        })
        .seal();
    ecs.execute_once(|ctx| {
        let e1 = ctx.create_entity();
        e1.add(A {});
        e1.destroy();
    });
    ecs.execute_once(|ctx| {
        ctx.send_signal(Signal { Value: 0 });
    });

    assert_eq!(0, *signals.lock().unwrap());
}

#[test]
fn destroyed_entity_removed_from_filter() {
    let signals = Rc::new(Mutex::new(0));
    let mut ecs = EcsContainer::create()
        .configure_in_test(|world| {
            let signals = signals.clone();
            world.add_entity_signal_handler::<Signal>("test", ecs_filter!(A), move |_, _| {
                *signals.lock().unwrap() += 1;
            })
        })
        .seal();
    let (e1, _) = ecs.execute_once(|ctx| {
        let e1 = ctx.create_entity();
        e1.add(A {});
        e1.key()
    });
    let e1 = e1.unwrap();
    ecs.execute_once(|ctx| {
        let e1 = ctx.get_entity(e1).unwrap();
        e1.destroy();
    });
    ecs.execute_once(|ctx| {
        ctx.send_signal(Signal { Value: 0 });
    });

    assert_eq!(0, *signals.lock().unwrap());
}
