#![allow(dead_code)]
use std::marker::PhantomData;

pub trait TiVecKey {
    fn from_index(index: usize) -> Self;
    fn as_index(&self) -> usize;
}

pub struct TiVec<K, T> {
    pd: PhantomData<K>,
    inner: Vec<T>,
}

impl<K, T> Default for TiVec<K, T> {
    fn default() -> Self {
        TiVec::new()
    }
}

impl<K, T> TiVec<K, T> {
    pub(crate) fn new() -> TiVec<K, T> {
        TiVec {
            pd: Default::default(),
            inner: vec![],
        }
    }
}

impl<K, V> TiVec<K, V> {
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
    where
        K: TiVecKey,
    {
        self.inner.get_mut(key.as_index())
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: TiVecKey,
    {
        self.inner.get(key.as_index())
    }

    pub fn push_with_key(&mut self, f: impl FnOnce(&K) -> V) -> K
    where
        K: TiVecKey,
    {
        let key = K::from_index(self.inner.len());
        self.inner.push(f(&key));
        key
    }

    pub fn iter(&self) -> impl Iterator<Item=&V> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut V> {
        self.inner.iter_mut()
    }
}
