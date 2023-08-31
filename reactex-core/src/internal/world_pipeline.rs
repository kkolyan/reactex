use crate::internal::execution::ExecutionResult;
use crate::internal::world_core::World;
use crate::internal::world_volatile::VolatileWorld;
use log::trace;

macro_rules! step_simple__ {
    ($world:ident, $step:ident, $var:expr) => {
        let __step__ = |world: &mut $crate::World, _: &mut crate::ExecutionResult| {
            World::$step(world);
        };
        add_step_simple($world, stringify!($step), __step__, $var);
    };
}
macro_rules! step_resulted {
    ($world:ident, $step:ident, $var:expr) => {
        add_step_simple($world, stringify!($step), World::$step, $var);
    };
}

#[rustfmt::skip]
pub(crate) fn configure_pipeline(world: &mut World) {
    let mut invoke_signal_handler = 0;
    let mut schedule_destroyed_entities_component_removal = 0;
    let mut generate_disappear_events = 0;
    let mut flush_component_addition = 0;

    step_resulted!(world, invoke_signal_handler, &mut invoke_signal_handler);
    step_simple__!(world, schedule_destroyed_entities_component_removal, &mut schedule_destroyed_entities_component_removal);
    step_simple__!(world, generate_disappear_events, &mut generate_disappear_events);
    step_resulted!(world, invoke_disappear_handlers, &mut 0);
    step_simple__!(world, flush_component_removals, &mut 0);
    add_goto(world, "check_destroyed_entities_early",
        |world| !world.entities_to_destroy.before_disappear.is_empty(),
        schedule_destroyed_entities_component_removal,
    );
    add_goto( world, "check_removed_components_early",
        |world| !world.components_to_delete.before_disappear.is_empty(),
        generate_disappear_events,
    );
    step_simple__!(world, flush_entity_destroy_actions, &mut 0);
    step_simple__!(world, flush_entity_create_actions, &mut 0);
    step_simple__!(world, flush_component_addition, &mut flush_component_addition);
    step_simple__!(world, flush_component_modification, &mut 0);
    step_resulted!(world, invoke_appear_handlers, &mut 0);
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

fn add_step_simple(
    world: &mut World,
    name: &str,
    callback: fn(&mut World, &mut ExecutionResult),
    index: &mut usize,
) {
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
    Fn(fn(&mut World, &mut ExecutionResult)),
    Goto {
        condition: fn(&VolatileWorld) -> bool,
        destination_index: usize,
    },
}

pub(crate) struct ExecutionContext {
    pub(crate) cursor: usize,
}

pub(crate) fn execute_all_internal(world: &mut World) -> ExecutionResult {
    trace!("execute_all");
    let mut ctx = ExecutionContext { cursor: 0 };
    let mut result = ExecutionResult::new();
    while ctx.cursor < world.stable.sequence.len() {
        let step = &world.stable.sequence[ctx.cursor];
        trace!("executing step: {}", step.name);
        ctx.cursor += 1;
        match step.callback {
            PipelineStepImpl::Fn(callback) => {
                callback(world, &mut result);
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
    world.tx += 1;
    log_mdc::insert("tx", world.tx.to_string());
    result
}
