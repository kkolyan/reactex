use crate::component::StaticComponentType;
use crate::entity::EntityKey;
use crate::world_mod::world::ConfigurableWorld;
use crate::world_mod::world::StableWorld;
use crate::world_mod::world::VolatileWorld;
use crate::world_mod::world::World;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::RwLock;

#[derive(Copy, Clone)]
pub struct Ctx<'a, TSignal = ()> {
    pub signal: &'a TSignal,
    stable: &'a StableWorld,
    volatile: &'a RefCell<&'a mut VolatileWorld>,
}

pub struct Entity<'a> {
    pub key: EntityKey,
    stable: &'a StableWorld,
    volatile: &'a RefCell<&'a mut VolatileWorld>,
}

struct Mut<'a, TComponent> {
    pd: PhantomData<TComponent>,
    entity: Entity<'a>,
}

impl<'a, TSignal> Ctx<'a, TSignal> {
    pub fn new(
        signal: &'a TSignal,
        stable: &'a StableWorld,
        volatile: &'a RefCell<&'a mut VolatileWorld>,
    ) -> Ctx<'a, TSignal> {
        Ctx {
            signal,
            stable,
            volatile,
        }
    }

    pub fn create_entity<'b>(&'b self) -> Entity<'a> {
        let entity_storage = &mut self.stable.entity_storage.borrow_mut();
        let volatile_world = &mut self.volatile.borrow_mut();
        let key = volatile_world
            .deref_mut()
            .create_entity(entity_storage.deref_mut());
        Entity {
            key,
            stable: self.stable,
            volatile: self.volatile,
        }
    }

    pub fn get_entity<'b>(&'b self, key: EntityKey) -> Entity<'a> {
        Entity {
            key,
            stable: self.stable,
            volatile: self.volatile,
        }
    }

    pub fn send_signal<T: 'static>(&self, signal: T) {
        let volatile = &mut self.volatile.borrow_mut();
        volatile.signal(signal);
    }
}

impl EntityKey {
    // pub fn get_entity<'a>(self, ctx: &'a Ctx<'a>) -> Entity<'a> {
    //     ctx.get_entity(self)
    // }
}

impl<'a, TComponent: StaticComponentType> Deref for Mut<'a, TComponent> {
    type Target = TComponent;

    fn deref(&self) -> &Self::Target {
        self.entity.get::<TComponent>()
    }
}

impl<'a, TComponent: StaticComponentType> Mut<'a, TComponent> {
    pub fn modify(&self, change: impl FnOnce(&mut TComponent) + 'static) {
        self.entity.modify(change);
    }
}

impl<'a> Entity<'a> {
    pub fn destroy(self) {
        let entity_storage = &mut self.stable.entity_storage.borrow_mut();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .destroy_entity(self.key, entity_storage.deref_mut())
            .unwrap();
    }

    pub fn add<TComponent: StaticComponentType>(&self, value: TComponent) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .add_component(self.key, value, entity_storage.deref())
            .unwrap();
    }

    pub fn get<TComponent: StaticComponentType>(&self) -> &TComponent {
        self.stable
            .get_component::<TComponent>(self.key)
            .unwrap()
            .unwrap()
    }

    pub fn remove<TComponent: StaticComponentType>(&self) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .remove_component::<TComponent>(self.key, entity_storage.deref())
            .unwrap()
    }

    pub fn modify<TComponent: StaticComponentType>(
        &self,
        change: impl FnOnce(&mut TComponent) + 'static,
    ) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .modify_component::<TComponent>(self.key, change, entity_storage.deref())
            .unwrap()
    }
}

pub struct EcsContainerBuilder {
    world: ConfigurableWorld,
}

impl EcsContainerBuilder {
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

    pub fn execute_once(&mut self, actions: impl FnOnce(Ctx)) {
        let ctx = Ctx {
            signal: &(),
            stable: &self.world.stable,
            volatile: &RefCell::new(&mut self.world.volatile),
        };
        actions(ctx);
        self.world.execute_all();
    }
}

pub struct Module {
    tasks: Vec<Task>,
}

struct Task {
    action: fn(&mut ConfigurableWorld),
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
        static $ident: std::sync::RwLock<$crate::facade_2_0::Module> =
            std::sync::RwLock::new($crate::facade_2_0::Module::new());
    };
}

pub use crate::__ecs_module as ecs_module;
