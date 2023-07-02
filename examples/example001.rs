use std::thread::sleep;
use std::time::Duration;
use reactex::api::{ComponentTypeAware, ConfigurablePipeline, ExecutablePipeline, FilterKey, IntoFilterKey, WorldState, WorldWriter};
use reactex::stub::StubPipelineFactory;

struct Explosion {
    damage: f32,
    range: f32,
}

struct Health {
    health: f32,
}

struct Position {
    x: f32,
    y: f32,
}

struct Damage;

struct Update;

fn main() {
    let mut factory = StubPipelineFactory::new();
    factory.add_entity_signal_handler::<Update>(
        <(Explosion, Position)>::create_filter_key(&factory),
        |entity, _, state, writer| {
            let explosion = state.get_component::<Explosion>(entity).unwrap();
            let exp_pos = state.get_component::<Position>(entity).unwrap();
            state.query(
                &<(Health, Position)>::create_filter_key(state),
                |victim| {
                    let victim_pos = state.get_component::<Position>(victim).unwrap();
                    if (victim_pos.x - exp_pos.x).powi(2) + (victim_pos.y - exp_pos.y).powi(2) < explosion.range.powi(2) {
                        writer.update_component::<Health>(victim, |health| {
                            health.health -= explosion.damage;
                        });
                    }
                },
            );
        },
    );
    let mut pipeline = factory.complete_configuration();
    loop {
        pipeline.signal(Update {});
        pipeline.execute_all();
        sleep(Duration::from_secs_f32(1.0));
    }
}

// pub struct EcsCtx<'a> {
//     state: &'a WorldState,
//     writer: &'a mut WorldWriter,
// }
//
// pub struct Mut<T> {
//
// }
//
// #[on_signal]
// fn update_explosion(_: Update, explosion: &Explosion, exp_pos: Position, ctx: EcsCtx) {
//     #[query(ctx)]
//     |health: Mut<Health>, victim_pos: &Position| {
//         if victim_pos.distance(exp_pos) < explosion.range {
//             ctx.writer.update(health, |it| { it.health -= explosion.damage; });
//         }
//     }
// }