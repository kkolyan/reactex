// use crate::api::*;
//
// pub trait Sugar1<TSignal, C1> {
//     fn de_sugar(self) -> Box<dyn Fn(&TSignal, EntityCtx)>;
// }
//
//
// impl<TSignal, C1, Target> Sugar1<TSignal, C1> for Target
//     where Target: Fn(&TSignal, EntityCtx, &C1) + 'static
// {
//     fn de_sugar(self) -> Box<dyn Fn(&TSignal, EntityCtx)> {
//         Box::from(move |signal: &TSignal, ctx: EntityCtx| {
//             let c1 = ctx.state.get_component::<C1>(ctx.entity).unwrap();
//             self(signal, ctx, c1);
//         })
//     }
// }
//
// pub trait Sugar2<TSignal, C1, C2> {
//     fn de_sugar(self) -> Box<dyn Fn(&TSignal, EntityCtx)>;
// }
//
//
// impl<TSignal, C1, C2, Target> Sugar2<TSignal, C1, C2> for Target
//     where Target: Fn(&TSignal, EntityCtx, &C1, &C2) + 'static
// {
//     fn de_sugar(self) -> Box<dyn Fn(&TSignal, EntityCtx)> {
//         Box::from(move |signal: &TSignal, ctx: EntityCtx| {
//             let c1 = ctx.state.get_component::<C1>(ctx.entity).unwrap();
//             let c2 = ctx.state.get_component::<C2>(ctx.entity).unwrap();
//             self(signal, ctx, c1, c2);
//         })
//     }
// }
//
// pub trait DeSugar<T> {
//     fn into_de_sugar(self) -> T;
// }
//
// impl <TSignal, C1, C2, Target> DeSugar<Target>
// for Box<dyn Fn(&TSignal, EntityCtx, &C1, &C2)>
//     where Target: Fn(&TSignal, EntityCtx) + 'static
// {
//     fn into_de_sugar(self) -> Target {
//         todo!()
//     }
// }
