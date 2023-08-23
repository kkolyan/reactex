use crate::ctx::Ctx;
use crate::internal::execution::invoke_user_code;
use crate::internal::execution::UserCode;
use crate::module::Module;
use crate::ConfigurableWorld;
use crate::World;
use log::trace;
use std::panic::UnwindSafe;
use std::sync::RwLock;

pub struct EcsContainerBuilder {
    world: ConfigurableWorld,
}

impl EcsContainerBuilder {
    pub fn configure_in_test(
        mut self,
        actions: impl FnOnce(&mut ConfigurableWorld),
    ) -> EcsContainerBuilder {
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

    pub fn execute_once<T>(&mut self, actions: impl (FnOnce(Ctx) -> T) + UnwindSafe) -> T {
        trace!("execute_once");
        let stable = &mut self.world.stable;
        let mut result = None;
        invoke_user_code(
            &mut self.world.volatile,
            stable,
            &mut self.world.entity_storage,
            "execute_once",
            [],
            [UserCode::new(actions)],
            |r| result = Some(r),
            &(),
        );
        self.world.execute_all();
        result.unwrap()
    }
}
