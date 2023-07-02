use crate::api::*;

pub trait Sugar1<R: WorldState, W: WorldWriter, TSignal, C1> {
    fn into_entity_signal_handler(self) -> Box<dyn Fn(&TSignal, Entity, &R, &mut W)>;
}


impl<R, W, TSignal, C1, Target> Sugar1<R, W, TSignal, C1> for Target
    where R: WorldState,
          W: WorldWriter,
          Target: Fn(&TSignal, Entity, &C1, &R, &mut W) + 'static
{
    fn into_entity_signal_handler(self) -> Box<dyn Fn(&TSignal, Entity, &R, &mut W)> {
        Box::from(move |signal: &TSignal, entity: Entity, state: &R, writer: &mut W| {
            let c1 = state.get_component::<C1>(entity).unwrap();
            self(signal, entity, c1, state, writer);
        })
    }
}

pub trait Sugar2<R: WorldState, W: WorldWriter, TSignal, C1, C2> {
    fn into_entity_signal_handler(self) -> Box<dyn Fn(&TSignal, Entity, &R, &mut W)>;
}


impl<R, W, TSignal, C1, C2, Target> Sugar2<R, W, TSignal, C1, C2> for Target
    where R: WorldState,
          W: WorldWriter,
          Target: Fn(&TSignal, Entity, &C1, &C2, &R, &mut W) + 'static
{
    fn into_entity_signal_handler(self) -> Box<dyn Fn(&TSignal, Entity, &R, &mut W)> {
        Box::from(move |signal: &TSignal, entity: Entity, state: &R, writer: &mut W| {
            let c1 = state.get_component::<C1>(entity).unwrap();
            let c2 = state.get_component::<C2>(entity).unwrap();
            self(signal, entity, c1, c2, state, writer);
        })
    }
}