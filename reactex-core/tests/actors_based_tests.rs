#![allow(non_snake_case)]

use std::any::Any;
use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use std::collections::VecDeque;
use std::fs;
use std::mem;
use std::ops::DerefMut;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::process::abort;
use std::sync::{Mutex, PoisonError};
use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;

use ctor::ctor;
use log::{error, warn};
use log::info;
use log::trace;
use rand::prelude::StdRng;
use rand::SeedableRng;
use syn::parse_str;
use syn::Item;
use syn::__private::ToTokens;
use to_vec::ToVec;

use reactex_core::{ecs_filter, EntityKey};
use reactex_core::panic_hook::catch_unwind_detailed;
use reactex_core::ConfigurableWorld;
use reactex_core::Ctx;
use reactex_core::EcsContainer;
use reactex_core::ExecutionError;
use reactex_core::World;
use reactex_macro::EcsComponent;

type ActorTestResult = Result<(), &'static str>;

struct ActorTemplate {
    name: String,
    params: Box<dyn (Fn(&mut StdRng) -> Box<dyn Any + RefUnwindSafe + UnwindSafe>) + Send>,
    actions: Vec<Box<dyn Fn(Ctx, &mut dyn Any) + Send + RefUnwindSafe>>,
    setup: fn(&mut ConfigurableWorld),
    action_names: Box<[&'static str]>,
}

struct Actor {
    template: ActorTemplate,
    last_param: Option<Box<dyn Any + RefUnwindSafe + UnwindSafe>>,
    last_action_index: Option<usize>,
    completed_iterations: usize,
    errors: VecDeque<ExecutionError>,
}

struct Context {
    expected_tests: usize,
    actors: Vec<ActorTemplate>,
    runtime_spawned: bool,
}

static CTX: Mutex<Option<Context>> = Mutex::new(None);
static RESULTS: Mutex<Option<HashMap<String, VecDeque<String>>>> = Mutex::new(None);

#[ctor]
fn init_logging() {
    let _ = log4rs::init_file("tests/log4rs.test.yaml", Default::default());
}

fn join_as_actor<T: RefUnwindSafe + UnwindSafe + 'static>(
    setup: fn(&mut ConfigurableWorld),
    steps: Vec<fn(Ctx, &mut T)>,
    next_instance: fn(&mut StdRng) -> T,
) -> ActorTestResult {
    let frame = Backtrace::capture()
        .to_string()
        .split('\n')
        .map(|it| it.to_string())
        // skip paths
        .filter(|it| !it.trim_start().starts_with("at "))
        // skip backtrace frames
        .skip_while(|it| !it.contains("join_as_actor"))
        // skip this method
        .nth(1)
        .unwrap();
    let name = frame.split("::").last().unwrap().to_string();
    info!("join_as_actor {}", name);
    {
        trace!("trying to lazy initializing context");
        let ctx = &mut CTX.lock().unwrap();
        let ctx = ctx.deref_mut();
        if ctx.is_none() {
            trace!("initializing context");
            trace!("current dir is {:?}", fs::canonicalize("."));
            let me_src = fs::read_to_string("tests/actors_based_tests.rs").unwrap();
            let me: syn::File = parse_str(me_src.as_str()).unwrap();
            let expected_tests = me
                .items
                .iter()
                .filter_map(|it| match it {
                    Item::Fn(it) => Some(it),
                    _ => None,
                })
                .filter(|it| {
                    it.attrs.iter().any(|it| {
                        let string = it.to_token_stream().to_string();
                        string == "# [test]"
                    })
                })
                .count();
            assert!(expected_tests > 0, "no tests found");
            info!("tests found: {}", expected_tests);
            *ctx = Some(Context {
                expected_tests,
                actors: vec![],
                runtime_spawned: false,
            });
        }
        trace!("push actor to shared queue");
        let ctx = ctx.as_mut().unwrap();
        assert!(!ctx.runtime_spawned, "test is too late");
        let action_names = steps
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let x1: &'static str = format!("{}[{}]", name.as_str(), i).leak();
                x1
            })
            .to_vec()
            .into_boxed_slice();
        ctx.actors.push(ActorTemplate {
            name: name.clone(),
            params: Box::new(move |rng| Box::new(next_instance(rng))),
            actions: steps
                .into_iter()
                .map(|it| {
                    let x: Box<dyn Fn(Ctx, &mut dyn Any) + Send + RefUnwindSafe> =
                        Box::new(move |a, b: &mut dyn Any| it(a, b.downcast_mut::<T>().unwrap()));
                    x
                })
                .to_vec(),
            action_names,
            setup,
        });
    }
    trace!("awaiting for all to be started");
    for i in 0.. {
        trace!("awaiting for all to be started: iteration {:03}", i);
        let ctx = &mut CTX.lock().unwrap();
        let ctx = ctx.as_mut().unwrap();
        if ctx.runtime_spawned {
            break;
        }
        if ctx.actors.len() >= ctx.expected_tests {
            info!("spawning run_actors");
            spawn(run_actors);
            ctx.runtime_spawned = true;
        }
        trace!("awaiting for all to be started: sleeping {:03}", i);
        sleep(Duration::from_millis(100));
    }
    trace!("awaiting to results");
    let mut fail = false;
    for i in 0.. {
        trace!("awaiting to results: iteration {:03}", i);
        let results = &mut RESULTS.lock().unwrap();
        if results.is_none() {
            *results.deref_mut() = Some(HashMap::new());
        }
        let results = results.as_mut().unwrap();
        trace!("taking results for {:?}", &name);
        if let Some(mut result) = results.remove(&name) {
            fail = !result.is_empty();
            while let Some(result) = result.pop_front() {
                println!("{}", result);
            }
            break;
        }
        trace!("awaiting to results: sleeping {:03}", i);
        sleep(Duration::from_millis(100));
    }
    match fail {
        true => Err("see logs for details"),
        false => Ok(()),
    }
}

fn run_actors() {
    let result = catch_unwind_detailed(|| {
        let mut actors = {
            let ctx = &mut CTX.lock().unwrap();
            let ctx = ctx.deref_mut().as_mut().unwrap();
            mem::take(&mut ctx.actors)
        };
        actors.sort_by_key(|it| it.name.clone());
        let mut actors = actors
            .into_iter()
            .map(|template| {
                Actor {
                    template,
                    last_param: None,
                    last_action_index: None,
                    completed_iterations: 0,
                    errors: Default::default(),
                }
            })
            .to_vec();

        let mut ecs = EcsContainer::create();
        for actor in actors.iter() {
            ecs = ecs.configure_in_test(|world| {
                (actor.template.setup)(world);
            });
        }
        let mut rng = StdRng::seed_from_u64(42);
        let mut ecs = ecs.seal();

        loop {
            let done = actors
                .iter()
                .all(|it| !it.errors.is_empty() || it.completed_iterations > ITERATIONS);
            if done {
                break;
            }
            for actor in actors.iter_mut() {
                if !actor.errors.is_empty() {
                    continue;
                }
                if actor.last_action_index.is_none() {
                    actor.last_action_index = Some(0);
                    actor.last_param = Some((actor.template.params)(&mut rng));
                }
                if actor.last_action_index.unwrap() >= actor.template.actions.len() {
                    actor.last_action_index = Some(0);
                    actor.last_param = Some((actor.template.params)(&mut rng));
                    actor.completed_iterations += 1;
                }
                let action = actor
                    .template
                    .actions
                    .get(actor.last_action_index.unwrap())
                    .unwrap();
                let x = actor.last_param.take().unwrap();
                let action_name = actor.template.action_names.get(actor.last_action_index.unwrap()).unwrap().clone();
                let (ret, result) = ecs.execute_once(action_name, move |ctx| {
                    let mut x = x;
                    action(ctx, x.as_mut());
                    x
                });
                actor.last_param = ret;
                *actor.last_action_index.as_mut().unwrap() += 1;
                actor.errors.extend(result.errors);
            }
        }
        for actor in actors {
            let mut mutex_guard = RESULTS.lock().unwrap();
            let guard = mutex_guard.as_mut().unwrap();
            trace!("storing results for {:?}", actor.template.name);
            guard.insert(
                actor.template.name,
                VecDeque::from_iter(actor.errors.iter().map(|it| it.to_string())),
            );
        }
    });
    if let Err(err) = result {
        error!("failed to run actors: {}", err);
        abort();
    }
}

const ITERATIONS: usize = 10;

#[derive(EcsComponent)]
pub struct A {}

#[derive(EcsComponent)]
pub struct B {}

#[test]
fn just_add() -> ActorTestResult {
    World::register_query(ecs_filter!(A));
    join_as_actor(
        |_w| {},
        vec![
            |ctx, seed| *seed = Some(ctx.create_entity().add(A {}).key()),
            |ctx, _seed| {
                ctx.query(ecs_filter!(A)).for_each(|e| {
                    e.get::<A>().unwrap();
                });
            },
        ],
        |_rng| None,
    )
}
#[test]
fn add_then_delete_b() -> ActorTestResult {
    World::register_query(ecs_filter!(B));
    join_as_actor(
        |_w| {},
        vec![
            |ctx, seed| *seed = Some(ctx.create_entity().add(B {}).key()),
            |ctx, seed| {
                let es = ctx
                    .query(ecs_filter!(B))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let e = es.get(0).unwrap();
                e.get::<B>().unwrap();
                assert_eq!(e.key(), seed.unwrap());
            },
            |ctx, seed| {
                let seed = seed.unwrap();
                ctx.get_entity(seed).unwrap().destroy();
                ctx.query(ecs_filter!(B)).for_each(|e| {
                    e.get::<B>().unwrap();
                });
            },
        ],
        |_rng| None,
    )
}

#[test]
fn add_then_delete() -> ActorTestResult {
    World::register_query(ecs_filter!(A));
    join_as_actor(
        |_w| {},
        vec![
            |ctx, seed| *seed = Some(ctx.create_entity().add(A {}).key()),
            |ctx, seed| {
                let es = ctx
                    .query(ecs_filter!(A))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let e = es.get(0).unwrap();
                e.get::<A>().unwrap();
            },
            |ctx, seed| {
                let seed = seed.unwrap();
                ctx.get_entity(seed).unwrap().destroy();
                ctx.query(ecs_filter!(A)).for_each(|e| {
                    e.get::<A>().unwrap();
                });
            },
        ],
        |_rng| None,
    )
}

#[test]
fn add_then_keep() -> ActorTestResult {
    World::register_query(ecs_filter!(A));
    join_as_actor(
        |_w| {},
        vec![
            |ctx, seed| *seed = Some(ctx.create_entity().add(A {}).key()),
            |ctx, seed| {
                let es = ctx
                    .query(ecs_filter!(A))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let e = es.get(0).unwrap();
                e.get::<A>().unwrap();
            },
        ],
        |_rng| None,
    )
}

#[test]
fn add_then_keep_b() -> ActorTestResult {
    World::register_query(ecs_filter!(B));
    join_as_actor(
        |_w| {},
        vec![
            |ctx, seed| *seed = Some(ctx.create_entity().add(B {}).key()),
            |ctx, seed| {
                let es = ctx
                    .query(ecs_filter!(B))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let e = es.get(0).unwrap();
                e.get::<B>().unwrap();
            },
        ],
        |_rng| None,
    )
}

#[test]
fn add_then_delete_AB() -> ActorTestResult {
    World::register_query(ecs_filter!(A));
    World::register_query(ecs_filter!(B));
    World::register_query(ecs_filter!(A, B));
    join_as_actor(
        |_w| {},
        vec![
            |ctx, seed| *seed = Some(ctx.create_entity().key()),
            |ctx, seed| {
                ctx.get_entity(seed.unwrap()).unwrap().add(A {});
            },
            |ctx, seed| {
                ctx.get_entity(seed.unwrap()).unwrap().add(B {});
            },
            |ctx, seed| {
                let es = ctx
                    .query(ecs_filter!(A, B))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let es = ctx
                    .query(ecs_filter!(A))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let es = ctx
                    .query(ecs_filter!(B))
                    .filter(|it| it.key() == seed.unwrap())
                    .to_vec();
                assert_eq!(1, es.len());
                let e = es.get(0).unwrap();
                e.get::<A>().unwrap();
                e.get::<B>().unwrap();
            },
            |ctx, seed| {
                let seed = seed.unwrap();
                ctx.get_entity(seed).unwrap().destroy();
            },
        ],
        |_rng| None,
    )
}


#[test]
fn on_appear() -> ActorTestResult {
    World::register_query(ecs_filter!(A));
    static APPEARED: Mutex<Vec<EntityKey>> = Mutex::new(Vec::new());

    join_as_actor(
        |w| w.add_appear_handler("on_appear_001", ecs_filter!(A), |_ctx, e| {
            APPEARED.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(e);
        }),
        vec![
            |ctx, s| {
                *s = Some(ctx.query(ecs_filter!(A))
                    .map(|it| it.key())
                    .collect::<HashSet<_>>());
                APPEARED.lock().unwrap().clear();
            },
            |ctx, s| {
                let prev = s.take().unwrap();
                let new = ctx.query(ecs_filter!(A))
                    .map(|it| it.key())
                    .filter(|it| !prev.contains(it))
                    .collect::<HashSet<_>>();
                assert_eq!(new, HashSet::from_iter(mem::take(APPEARED.lock().unwrap().deref_mut())));
            },
        ],
        |_rng| None
    )
}
