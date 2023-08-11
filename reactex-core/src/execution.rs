use crate::world::World;

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
