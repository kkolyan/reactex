use crate::pools::AbstractPool;
use crate::pools::PoolKey;
use std::marker::PhantomData;

pub trait AbstractPoolPump<TKeySrc, TKeyDst> {
    fn do_move(
        &self,
        src: &mut dyn AbstractPool<TKeySrc>,
        dst: &mut dyn AbstractPool<TKeyDst>,
        key: &TKeySrc,
    ) -> TKeyDst;
}

pub struct SpecificPoolPump<TKeySrc, TKeyDst, TValue> {
    pd: PhantomData<(TKeySrc, TKeyDst, TValue)>,
}

impl<TKeySrc, TKeyDst, TValue> Default for SpecificPoolPump<TKeySrc, TKeyDst, TValue> {
    fn default() -> Self {
        Self {
            pd: Default::default(),
        }
    }
}

impl<TKeySrc: PoolKey, TKeyDst: PoolKey, TValue: 'static> AbstractPoolPump<TKeySrc, TKeyDst>
    for SpecificPoolPump<TKeySrc, TKeyDst, TValue>
{
    fn do_move(
        &self,
        src: &mut dyn AbstractPool<TKeySrc>,
        dst: &mut dyn AbstractPool<TKeyDst>,
        key: &TKeySrc,
    ) -> TKeyDst {
        let value = src
            .specializable_mut()
            .try_specialize::<TValue>()
            .unwrap()
            .del_and_get(key)
            .unwrap();
        dst.specializable_mut()
            .try_specialize::<TValue>()
            .unwrap()
            .add(value)
    }
}
