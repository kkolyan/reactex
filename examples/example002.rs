use std::thread::sleep;
use std::time::Duration;
use reactex::api::{ComponentTypeAware, ConfigurablePipeline, Entity, ExecutablePipeline, FilterKey, IntoFilterKey, WorldState, WorldWriter};
use reactex::gen::Sugar2;
use reactex::stub::{StubPipelineFactory, StubState, StubWriter};

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
        update_explosion.into_entity_signal_handler(),
    );
    let mut pipeline = factory.complete_configuration();
    loop {
        pipeline.signal(Update {});
        pipeline.execute_all();
        sleep(Duration::from_secs_f32(1.0));
    }
}


fn update_explosion(_: &Update, entity: Entity, explosion: &Explosion, exp_pos: &Position, state: &impl WorldState, writer: &mut impl WorldWriter) {
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
}
