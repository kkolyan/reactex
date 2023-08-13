use log::trace;
use crate::world_mod::world::World;

pub struct Step {
    pub name: String,
    pub callback: StepImpl,
}

pub enum StepImpl {
    Fn(fn(&mut World)),
    Goto {
        condition: fn(&World) -> bool,
        destination_index: usize,
    },
}

pub struct ExecutionContext {
    pub cursor: usize,
}

trait StepAction {
    fn execute(world: &mut World, ctx: &mut ExecutionContext);
    fn get_name() -> &'static str;
}

struct InvokeSignalHandler;

impl StepAction for InvokeSignalHandler {
    fn execute(_world: &mut World, _ctx: &mut ExecutionContext) {}

    fn get_name() -> &'static str {
        "InvokeSignalHandler"
    }
}

impl World {
    pub fn execute_all(&mut self) {
        trace!("execute_all");
        let mut ctx = ExecutionContext { cursor: 0 };
        while ctx.cursor < self.sequence.len() {
            let step = &self.sequence[ctx.cursor];
            trace!("executing step: {}", step.name);
            ctx.cursor += 1;
            match step.callback {
                StepImpl::Fn(callback) => {
                    callback(self);
                }
                StepImpl::Goto {
                    condition,
                    destination_index,
                } => {
                    if condition(self) {
                        ctx.cursor = destination_index;
                    }
                }
            }
        }
    }
}
