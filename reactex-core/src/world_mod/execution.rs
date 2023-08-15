use crate::world_mod::world::{VolatileWorld, World};
use log::trace;

pub struct Step {
    pub name: String,
    pub callback: StepImpl,
}

pub enum StepImpl {
    Fn(fn(&mut World)),
    Goto {
        condition: fn(&VolatileWorld) -> bool,
        destination_index: usize,
    },
}

pub struct ExecutionContext {
    pub cursor: usize,
}

trait StepAction {
    fn execute(world: &mut VolatileWorld, ctx: &mut ExecutionContext);
    fn get_name() -> &'static str;
}

struct InvokeSignalHandler;

impl StepAction for InvokeSignalHandler {
    fn execute(_world: &mut VolatileWorld, _ctx: &mut ExecutionContext) {}

    fn get_name() -> &'static str {
        "InvokeSignalHandler"
    }
}

impl World {
    pub fn execute_all(&mut self) {
        trace!("execute_all");
        let mut ctx = ExecutionContext { cursor: 0 };
        while ctx.cursor < self.stable.sequence.len() {
            let step = &self.stable.sequence[ctx.cursor];
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
                    if condition(&mut self.volatile) {
                        ctx.cursor = destination_index;
                    }
                }
            }
        }
    }
}
