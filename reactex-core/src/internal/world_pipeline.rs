use crate::internal::world_core::World;
use crate::internal::world_volatile::VolatileWorld;
use log::trace;

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

#[rustfmt::skip]
pub(crate) fn configure_pipeline(world: &mut World) {
    let mut invoke_signal_handler = 0;
    let mut schedule_destroyed_entities_component_removal = 0;
    let mut generate_disappear_events = 0;
    let mut flush_component_addition = 0;

    add_step_simple(world, stringify!( invoke_signal_handler ), World::invoke_signal_handler, &mut invoke_signal_handler);
    step_simple_a!(world, schedule_destroyed_entities_component_removal);
    step_simple_a!(world, generate_disappear_events);
    step_simple_b!(world, invoke_disappear_handlers);
    step_simple_b!(world, flush_component_removals);
    add_goto(world, "check_destroyed_entities_early",
        |world| !world.entities_to_destroy.before_disappear.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto( world, "check_removed_components_early",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    step_simple_b!(world, flush_entity_destroy_actions);
    step_simple_b!(world, flush_entity_create_actions);
    step_simple_a!(world, flush_component_addition);
    step_simple_b!(world, flush_component_modification);
    step_simple_b!(world, invoke_appear_handlers);
    add_goto(world, "check_destroyed_entities_late",
        |world| !world.entities_to_destroy.before_disappear.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto(world,"check_removed_components_late",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    add_goto(world, "check_added_components",
        |world| !world.components_to_add.is_empty(),
        flush_component_addition,
    );
    add_goto( world, "check_signals",
        |world| !world.signal_queue.signals.is_empty(),
        invoke_signal_handler,
    );
}

fn add_step_simple(world: &mut World, name: &str, callback: fn(&mut World), index: &mut usize) {
    *index = world.stable.sequence.len();
    world.stable.sequence.push(PipelineStep {
        name: name.to_string(),
        callback: PipelineStepImpl::Fn(callback),
    })
}

fn add_goto(
    world: &mut World,
    name: &str,
    condition: fn(&VolatileWorld) -> bool,
    destination_index: usize,
) {
    world.stable.sequence.push(PipelineStep {
        name: name.to_string(),
        callback: PipelineStepImpl::Goto {
            condition,
            destination_index,
        },
    })
}

pub(crate) struct PipelineStep {
    pub(crate) name: String,
    pub(crate) callback: PipelineStepImpl,
}

pub(crate) enum PipelineStepImpl {
    Fn(fn(&mut World)),
    Goto {
        condition: fn(&VolatileWorld) -> bool,
        destination_index: usize,
    },
}

pub(crate) struct ExecutionContext {
    pub(crate) cursor: usize,
}

pub(crate) fn execute_all_internal(world: &mut World) {
    trace!("execute_all");
    let mut ctx = ExecutionContext { cursor: 0 };
    while ctx.cursor < world.stable.sequence.len() {
        let step = &world.stable.sequence[ctx.cursor];
        trace!("executing step: {}", step.name);
        ctx.cursor += 1;
        match step.callback {
            PipelineStepImpl::Fn(callback) => {
                callback(world);
            }
            PipelineStepImpl::Goto {
                condition,
                destination_index,
            } => {
                if condition(&world.volatile) {
                    ctx.cursor = destination_index;
                }
            }
        }
    }
}
