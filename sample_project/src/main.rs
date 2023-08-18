use reactex_core::facade_2_0::ecs_module;
use reactex_core::facade_2_0::Ctx;
use reactex_core::facade_2_0::EcsContainer;
use reactex_core::reactex_macro::on_appear;
use reactex_core::reactex_macro::on_disappear;
use reactex_core::reactex_macro::on_signal;
use reactex_core::reactex_macro::on_signal_global;
use reactex_core::reactex_macro::EcsComponent;

ecs_module!(DEMO);

struct Seed;

#[derive(EcsComponent, Debug)]
struct A {}


#[on_signal_global(DEMO)]
fn system1(ctx: Ctx<Seed>) {}
#[on_signal(DEMO)]
fn system2(ctx: Ctx<Seed>) {}
#[on_appear(DEMO)]
fn system3(ctx: Ctx) {}
#[on_disappear(DEMO)]
fn system4(ctx: Ctx) {}



fn main() {
    let mut ecs = EcsContainer::create().add_module(&DEMO).seal();

    ecs.execute_once(|ctx| {
        let entity = ctx.create_entity();
        entity.add(A {});
    });
    ecs.execute_once(|ctx| {
        ctx.send_signal(Seed);
    });
}
