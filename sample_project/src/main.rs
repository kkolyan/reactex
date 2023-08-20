#![allow(clippy::explicit_auto_deref)]
#![feature(proc_macro_hygiene)]

use reactex_core::entity::EntityKey;
use reactex_core::facade_2_0::ecs_module;
use reactex_core::facade_2_0::Ctx;
use reactex_core::facade_2_0::EcsContainer;
use reactex_core::facade_2_0::Entity;
use reactex_core::facade_2_0::Mut;
use reactex_core::facade_2_0::UncommittedEntity;
use reactex_core::reactex_macro::enable_queries;
use reactex_core::reactex_macro::on_appear;
use reactex_core::reactex_macro::on_disappear;
use reactex_core::reactex_macro::on_signal;
use reactex_core::reactex_macro::on_signal_global;
use reactex_core::reactex_macro::query;
use reactex_core::reactex_macro::EcsComponent;

// all ECS systems are bound to some module ID. this ID could be used to register all associated
// ECS systems at once at ECS initialization.
// under the hood that's RwLock<Vec<fn(...)+metadata>>
ecs_module!(DEMO);

// most of ECS systems invocation are bound to signals. the most obvious is frame update signal.
// other application lifecycle phases could be used.
struct SomeSignal;

struct AnotherSignal;

// component type is a component identifier. it's required to derive EcsComponent (Debug)
#[derive(EcsComponent)]
struct A {}

#[derive(EcsComponent)]
struct B {}

#[derive(EcsComponent)]
struct C {
    value: i32,
}

// attribute macro generates code to associate function reference and its metadata with a DEMO module
// module may be defined in other package if needed.
#[on_signal_global(DEMO)]
fn system1(ctx: Ctx<SomeSignal>) {
    // this code invoked one time if corresponding signal issued.
    // global means it is not associated with any entity

    // signal instance can be read
    let _signal_payload: &SomeSignal = ctx.signal;

    // entity can be created here.
    let entity: UncommittedEntity = ctx.create_entity();

    // returned handle is not just ID - it has convenient methods

    // deferred component addition
    entity.add(A {});

    // EntityKey is identifier, that is safe to store inside component fields or somewhere else
    // and safely retrieve Entity later
    let _entity_key: EntityKey = entity.key();

    // entity will be destroyed immediately, because it is not committed
    entity.destroy();

    // that's all what can be done for new entity.

    // but you can send signals from here
    ctx.send_signal(AnotherSignal)
}

#[on_signal(DEMO)]
fn system2(_ctx: Ctx<AnotherSignal>, _a: &A) {
    // invoked once per entity with A component. component is read-only..
}

#[on_signal(DEMO)]
fn system2a(_ctx: Ctx<SomeSignal>, entity: Entity, _a: &A) {
    // invoked once per entity with A component too, but Entity object is available.

    // another components can be queried for read
    let _b: Option<&B> = entity.get::<B>();

    // deferred removal of component
    entity.remove::<A>();

    // deferred destroy of entity
    entity.destroy();
}

#[on_signal(DEMO)]
fn system2b(_ctx: Ctx<SomeSignal>, c: Mut<C>) {
    // invoked once per entity with C component, but component is modifiable

    // Mut<_> is Deref, sou you can read a component through
    let _c_value: i32 = c.value;
    let _c_as_ref: &C = &*c;

    // deferred mutation of component (type annotation at lambda is not required)
    c.modify(|a: &mut C| a.value = 42);
}

#[on_appear(DEMO)]
fn system3b(_ctx: Ctx, _entity: Entity, _a: &A, _b: &B) {
    // called when FULL combination of A and B components appears on some entity. combination is
    // said to appear hwn the last component of criteria has been added
}

#[on_disappear(DEMO)]
fn system4b(_ctx: Ctx, _entity: Entity, _a: &A, _b: &B) {
    // called before the disappear of the A+B combination on some entity. removal of one of
    // components is sufficient for the combination to disappear.
}

#[on_appear(DEMO)]
fn system3(_ctx: Ctx, _entity: Entity) {
    // called for each created entity
}

#[on_disappear(DEMO)]
fn system4(_ctx: Ctx, _entity: Entity) {
    // called for each destroyed entity (before its destroy)
}

#[on_appear(DEMO)]
fn system5(_entity: Entity) {
    // _ctx is optional for all non-signal events handlers.
}

#[on_signal(DEMO)]
fn system6(entity: Entity, _a: &A, _ctx: Ctx<SomeSignal>) {
    // argument order doesn't matter
}

struct D {
    x: i32,
}

// #[enable_queries] in method is needed for #[query(ctx)] to work (it does all transformations and query is just a marker attribute).
#[enable_queries]
fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let mut ecs = EcsContainer::create()
        // register your module (or N of them)
        .add_module(&DEMO)
        // end configuration and begin work. no configuration is allowed anymore
        .seal();

    // do some actions on the top level (outside of event callbacks)
    ecs.execute_once(|ctx| {
        let entity = ctx.create_entity();
        entity.add(A {});
    });

    ecs.execute_once(|ctx| {
        // orchestrate application using signals
        ctx.send_signal(SomeSignal);
    });

    ecs.execute_once(|ctx| {
        let mut d = D { x: 0 };

        // query-annotated closure  is invoked right here for all matched entities.
        // matching is performed by the same rules as for signal handlers with the exception
        // that there is not any signal here.
        // note that query reads only committed changes.

        #[query(ctx)]
        |a: &A, c: &C, entity: Entity| {
            d.x = c.value * 4;
        };

        #[query(ctx)]
        |c: Mut<C>| {
            c.modify(move |c| c.value = d.x * 10);
        };
    });
}
