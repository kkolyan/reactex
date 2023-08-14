use crate::world_mod::execution::Step;
use crate::world_mod::execution::StepImpl;
use crate::world_mod::world::World;

macro_rules! step_simple_a {
    ($world:ident, $step:ident) => {
        add_step_simple($world, stringify!($step), World::$step, &mut $step);
    };
}
macro_rules! step_simple_b {
    ($world:ident, $step:ident) => {
        add_step_simple($world, stringify!($step), World::$step, &mut 0);
    };
}

pub fn configure_pipeline(world: &mut World) {
    let mut invoke_signal_handler = 0;
    let mut schedule_destroyed_entities_component_removal = 0;
    let mut generate_disappear_events = 0;
    let mut flush_component_addition = 0;

    step_simple_a!(world, invoke_signal_handler);
    step_simple_a!(world, schedule_destroyed_entities_component_removal);
    step_simple_a!(world, generate_disappear_events);
    step_simple_b!(world, invoke_disappear_handlers);
    step_simple_b!(world, flush_component_removals);
    add_goto(
        world,
        "check_destroyed_entities_early",
        |world| !world.entities_to_destroy.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto(
        world,
        "check_removed_components_early",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    step_simple_b!(world, flush_entity_destroy_actions);
    step_simple_b!(world, flush_entity_create_actions);
    step_simple_a!(world, flush_component_addition);
    step_simple_b!(world, invoke_appear_handlers);
    add_goto(
        world,
        "check_destroyed_entities_late",
        |world| !world.entities_to_destroy.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto(
        world,
        "check_removed_components_late",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    add_goto(
        world,
        "check_added_components",
        |world| !world.components_to_add.is_empty(),
        flush_component_addition,
    );
    add_goto(
        world,
        "check_signals",
        |world| !world.signal_queue.signals.is_empty(),
        invoke_signal_handler,
    );
}

fn add_step_simple(world: &mut World, name: &str, callback: fn(&mut World), index: &mut usize) {
    *index = world.sequence.len();
    world.sequence.push(Step {
        name: name.to_string(),
        callback: StepImpl::Fn(callback),
    })
}

fn add_goto(
    world: &mut World,
    name: &str,
    condition: fn(&World) -> bool,
    destination_index: usize,
) {
    world.sequence.push(Step {
        name: name.to_string(),
        callback: StepImpl::Goto {
            condition,
            destination_index,
        },
    })
}
