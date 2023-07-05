#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

use std::sync::RwLock;

use reactex::api::*;
use reactex_macro::*;


struct Explosion {
    damage: f32,
    range: f32,
}

impl GetRef for Explosion {
    fn get() -> &'static Self {
        static x: Explosion = Explosion { damage: 0.0, range: 0.0 };
        &x
    }
}

struct Health {
    health: f32,
    status: Statuses,
}

impl Health {
    fn heal(&mut self) {

    }
}

struct Statuses {
    drunk: bool,
    dead: bool,
}

#[derive(Copy, Clone)]
struct Position {
    x: f32,
    y: f32,
}

impl GetRef for Position {
    fn get() -> &'static Self {
        static x: Position = Position { x: 0.0, y: 0.0 };
        &x
    }
}

impl GetRef for Health {
    fn get() -> &'static Self {
        static x: Health = Health { health: 0.0, status: Statuses { drunk: false, dead: false } };
        &x
    }
}

struct Damage;

struct Update;

fn main() {
    let mut factory = ConfigurablePipeline::new();
    // factory.add_entity_signal_handler::<Update>(
    //     <(Explosion, Position)>::create_filter_key(&factory),
    //     update_explosion,
    // );
    // // factory.add_entity_signal_handler::<Update>(
    // //     <(Explosion, Position)>::create_filter_key(&factory),
    // //     (|_: &Update, ctx: EntityCtx, c1: &Explosion, c2: &Position| {}).de_sugar(),
    // // );
    // let mut pipeline = factory.complete_configuration();
    // loop {
    //     pipeline.signal(Update {});
    //     pipeline.execute_all();
    //     sleep(Duration::from_secs_f32(1.0));
    // }

    update_explosion(Ctx {
        state: &WorldState {},
        changes: &mut WorldChanges {},
        signal: &Update {},
    }, Entity { index: 0, generation: 0 });
}

static DEMO: RwLock<Module> = RwLock::new(Module::new());

//noinspection DuplicatedCode
#[on_signal(DEMO)]
fn update_explosion(ctx: Ctx<Update>, explosion: &Explosion, exp_pos: Mut<Position>) {
    // with macros 2
    #[query(ctx)]
    |victim_pos: &Position, health: Mut<Health>| {
        if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
            health.modify(|it| it.health -= explosion.damage);
            health.modify(|it| it.heal());
            health.modify(|it| it.status.dead = true);
        }
    };
}

//noinspection DuplicatedCode
#[on_signal_global(DEMO)]
fn update_all(ctx: Ctx<Update>) {
}

//noinspection DuplicatedCode
#[on_appear(DEMO)]
fn on_explosion_appear(ctx: Ctx, explosion: &Explosion, exp_pos: Mut<Position>) {
}

//noinspection DuplicatedCode
#[on_disappear(DEMO)]
fn on_explosion_disappear(ctx: Ctx, explosion: &Explosion, exp_pos: Mut<Position>) {
}
