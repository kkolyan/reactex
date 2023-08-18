use crate::component::StaticComponentType;
use crate::entity::EntityKey;
use crate::entity::InternalEntityKey;
use crate::world_mod::world::ConfigurableWorld;
use crate::world_mod::world::EntityError;
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

#[derive(Copy, Clone)]
pub struct Entity<'a> {
    key: InternalEntityKey,
    stable: &'a StableWorld,
    volatile: &'a RefCell<&'a mut VolatileWorld>,
}

#[derive(Copy, Clone)]
pub struct UncommittedEntity<'a> {
    key: InternalEntityKey,
    stable: &'a StableWorld,
    volatile: &'a RefCell<&'a mut VolatileWorld>,
}

pub struct Mut<'a, TComponent> {
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

    pub fn create_entity<'b>(&'b self) -> UncommittedEntity<'a> {
        let entity_storage = &mut self.stable.entity_storage.borrow_mut();
        let volatile_world = &mut self.volatile.borrow_mut();
        let key = volatile_world
            .deref_mut()
            .create_entity(entity_storage.deref_mut());
        UncommittedEntity {
            key,
            stable: self.stable,
            volatile: self.volatile,
        }
    }

    pub fn get_entity<'b>(&'b self, key: EntityKey) -> Option<Entity<'a>> {
        let result = key.validate(
            self.stable.entity_storage.borrow().deref(),
            ValidateUncommitted::DenyUncommitted,
        );
        let entity_key = match result {
            Ok(it) => Some(it),
            Err(err) => match err {
                EntityError::NotExists => None,
                EntityError::NotCommitted => {
                    panic!("attempt to transform UncommittedEntity to Entity detected")
                }
                EntityError::IsStale => None,
            },
        }?;
        Some(Entity {
            key: entity_key,
            stable: self.stable,
            volatile: self.volatile,
        })
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
        self.entity.get::<TComponent>().unwrap()
    }
}

impl<'a, TComponent: StaticComponentType> Mut<'a, TComponent> {
    pub fn new(entity: Entity<'a>) -> Self {
        Mut {
            pd: Default::default(),
            entity,
        }
    }
    pub fn modify(&self, change: impl FnOnce(&mut TComponent) + 'static) {
        self.entity.modify(change);
    }
}

impl<'a> Entity<'a> {
    pub fn key(self) -> EntityKey {
        self.key.export()
    }

    pub fn destroy(self) {
        let entity_storage = &mut self.stable.entity_storage.borrow_mut();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .destroy_entity(self.key.export(), entity_storage.deref_mut())
            .unwrap();
    }

    pub fn add<TComponent: StaticComponentType>(&self, value: TComponent) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .add_component(self.key.export(), value, entity_storage.deref())
            .unwrap();
    }

    pub fn get<TComponent: StaticComponentType>(&self) -> Option<&TComponent> {
        self.stable
            .get_component::<TComponent>(self.key.export())
            .unwrap()
    }

    pub fn remove<TComponent: StaticComponentType>(&self) {
        let entity_storage = self.stable.entity_storage.borrow();
        let volatile_world = &mut self.volatile.borrow_mut();
        volatile_world
            .deref_mut()
            .remove_component::<TComponent>(self.key.export(), entity_storage.deref())
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
            .modify_component::<TComponent>(self.key.export(), change, entity_storage.deref())
            .unwrap()
    }
}

impl<'a> UncommittedEntity<'a> {
    pub fn key(&self) -> EntityKey {
        self.key.export()
    }

    pub fn destroy(self) {
        Entity {
            key: self.key,
            stable: self.stable,
            volatile: self.volatile,
        }
        .destroy();
    }

    pub fn add<TComponent: StaticComponentType>(&self, value: TComponent) {
        Entity {
            key: self.key,
            stable: self.stable,
            volatile: self.volatile,
        }
        .add(value);
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
use crate::world_mod::entity_storage::ValidateUncommitted;
