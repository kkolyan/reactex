use crate::ConfigurableWorld;

pub struct Module {
    pub(crate) tasks: Vec<Task>,
}

pub(crate) struct Task {
    pub(crate) action: fn(&mut ConfigurableWorld),
}

impl Module {
    pub const fn new() -> Module {
        Module { tasks: vec![] }
    }

    pub fn add_configurator(&mut self, action: fn(&mut ConfigurableWorld)) {
        self.tasks.push(Task { action });
    }
}

#[macro_export]
macro_rules! __ecs_module {
    ($ident:ident) => {
        static $ident: std::sync::RwLock<$crate::Module> =
            std::sync::RwLock::new($crate::Module::new());
    };
}

pub use crate::__ecs_module as ecs_module;
