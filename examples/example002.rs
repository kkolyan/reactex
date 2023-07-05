use std::convert::Into;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use reactex::api::{ComponentTypeAware, ConfigurablePipeline, Entity, ExecutablePipeline, FilterKey, IntoFilterKey, WorldState, WorldChanges, EntityCtx, GetRef, Module, Mut};
use reactex_macro::{on_signal, sub_query};

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
}

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
        static x: Health = Health { health: 0.0 };
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

    update_explosion(&Update {}, EntityCtx {
        state: &WorldState {},
        changes: &mut WorldChanges {},
        entity: Entity { index: 0, generation: 0 },
    })
}

static DEMO : RwLock<Module> = RwLock::new(Module::new());

#[on_signal(DEMO)]
fn update_explosion(_: &Update, explosion: &Explosion, exp_pos: Mut<Position>, ctx: EntityCtx) {

    // without macros
    ctx.state.query(
        &<(Health, Position)>::create_filter_key(ctx.state),
        |victim| {
            let victim_pos = ctx.state.get_component::<Position>(victim).unwrap();
            if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
                ctx.changes.update_component::<Health>(victim, |health| {
                    health.health -= explosion.damage;
                });
            }
        },
    );

    // with macros 1
    #[lookup(ctx)]
    |victim_pos: &Position, health: Mut<Health>| {
        if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {

            ctx.changes.update_mut_wrapper(&health, |health| { health.health -= explosion.damage; });
        }
    };

    // with macros 2
    sub_query!(ctx, |victim_pos: &Position, health: Mut<Health>| {
        if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
            #[update(ctx)]
            health.health -= explosion.damage;
        }
    });
}