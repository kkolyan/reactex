use std::any::Any;
use std::collections::VecDeque;
use std::marker::PhantomData;

pub struct SpecificPool<K, V> {
    pd: PhantomData<K>,
    buffer: Vec<Option<V>>,
    holes: VecDeque<usize>,
}

pub trait AbstractPool<K> {
    fn del(&mut self, key: &K);
    fn clear(&mut self);
    fn as_any_mut(&mut self) -> AnyPoolMut<K>;
    fn as_any(&self) -> AnyPool<K>;
}

pub trait PoolKey: 'static {
    fn as_usize(&self) -> usize;
    fn from_usize(value: usize) -> Self;
}

pub struct AnyPoolMut<'a, K> {
    pd: PhantomData<K>,
    any: &'a mut dyn Any,
}

pub struct AnyPool<'a, K> {
    pd: PhantomData<K>,
    any: &'a dyn Any,
}

impl<'a, K: 'static> AnyPoolMut<'a, K> {
    pub fn try_specialize<T: 'static>(self) -> Option<&'a mut SpecificPool<K, T>> {
        self.any.downcast_mut::<SpecificPool<K, T>>()
    }
}

impl<'a, K: 'static> AnyPool<'a, K> {
    pub fn try_specialize<T: 'static>(self) -> Option<&'a SpecificPool<K, T>> {
        self.any.downcast_ref::<SpecificPool<K, T>>()
    }
}

impl<K: PoolKey, V> SpecificPool<K, V> {
    pub fn new() -> Self {
        SpecificPool {
            pd: Default::default(),
            buffer: vec![],
            holes: Default::default(),
        }
    }

    pub fn add(&mut self, value: V) -> K {
        K::from_usize(match self.holes.pop_front() {
            None => {
                self.buffer.push(Some(value));
                self.buffer.len() - 1
            }
            Some(index) => {
                *self
                    .buffer
                    .get_mut(index)
                    .expect("WTF? holes contains index outside the bounds?") = Some(value);
                index
            }
        })
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = key.as_usize();
        self.buffer.get_mut(index).and_then(|it| it.as_mut())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let index = key.as_usize();
        self.buffer.get(index).and_then(|it| it.as_ref())
    }

    fn del_internal(&mut self, key: &K) -> Option<V> {
        let index = key.as_usize();
        if index < self.buffer.len() {
            if index == self.buffer.len() - 1 {
                self.buffer.remove(index)
            } else {
                let value = self
                    .buffer
                    .get_mut(index)
                    .expect("WTF? we've just checked index");
                self.holes.push_back(index);
                value.take()
            }
        } else {
            panic!("attempt to delete entry outside of the bounds")
        }
    }

    pub(crate) fn del_and_get(&mut self, key: &K) -> Option<V> {
        self.del_internal(key)
    }

    fn clear_internal(&mut self) {
        self.holes.clear();
        self.buffer.clear();
    }
}

impl<K: PoolKey + 'static, V: 'static> AbstractPool<K> for SpecificPool<K, V> {
    fn del(&mut self, key: &K) {
        self.del_internal(key);
    }

    fn clear(&mut self) {
        self.clear_internal();
    }

    fn as_any_mut(&mut self) -> AnyPoolMut<K> {
        AnyPoolMut {
            pd: Default::default(),
            any: self,
        }
    }

    fn as_any(&self) -> AnyPool<K> {
        AnyPool {
            pd: Default::default(),
            any: self,
        }
    }
}

impl PoolKey for usize {
    fn as_usize(&self) -> usize {
        *self
    }

    fn from_usize(value: usize) -> Self {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downcast_works() {
        let mut ints: Box<dyn AbstractPool<usize>> = Box::from(SpecificPool::<usize, i32>::new());
        let mut bools: Box<dyn AbstractPool<usize>> = Box::from(SpecificPool::<usize, bool>::new());
        assert!(ints.as_any_mut().try_specialize::<i32>().is_some());
        assert!(bools.as_any_mut().try_specialize::<i32>().is_none())
    }
}
