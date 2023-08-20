use crate::ctx::Ctx;
use crate::module::Module;
use crate::ConfigurableWorld;
use crate::World;
use std::cell::RefCell;
use std::sync::RwLock;
use log::trace;

pub struct EcsContainerBuilder {
    world: ConfigurableWorld,
}

impl EcsContainerBuilder {

    pub fn configure_in_test(mut self, actions: impl FnOnce(&mut ConfigurableWorld)) -> EcsContainerBuilder {
        actions(&mut self.world);
        self
    }

    pub fn add_module(mut self, module: &RwLock<Module>) -> EcsContainerBuilder {
        for task in module.read().unwrap().tasks.iter() {
            (task.action)(&mut self.world);
        }
        self
    }

    pub fn seal(self) -> EcsContainer {
        EcsContainer {
            world: self.world.seal(),
        }
    }
}

pub struct EcsContainer {
    world: World,
}

impl EcsContainer {
    pub fn create() -> EcsContainerBuilder {
        EcsContainerBuilder {
            world: ConfigurableWorld::new(),
        }
    }

    pub fn execute_once<T>(&mut self, actions: impl FnOnce(Ctx) -> T) -> T {
        trace!("execute_once");
        let ctx = Ctx {
            signal: &(),
            stable: &self.world.stable,
            volatile: &RefCell::new(&mut self.world.volatile),
        };
        let result = actions(ctx);
        self.world.execute_all();
        result
    }
}
