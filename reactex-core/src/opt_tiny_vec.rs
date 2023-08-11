use std::iter::Map;
use tinyvec::tiny_vec;
use tinyvec::TinyVec;
use tinyvec::TinyVecIterator;

const LIMIT: usize = 2;

#[derive(Debug, Clone)]
pub struct OptTinyVec<T> {
    inner: TinyVec<[Option<T>; LIMIT]>,
}

impl<T> Default for OptTinyVec<T> {
    fn default() -> Self {
        OptTinyVec { inner: tiny_vec!() }
    }
}

impl<T> OptTinyVec<T> {
    #[inline]
    pub fn single(value: T) -> OptTinyVec<T> {
        OptTinyVec {
            inner: tiny_vec!([Option<T>; LIMIT] => Some(value)),
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.inner.push(Some(value));
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.inner.iter().map(|it| it.as_ref().unwrap())
    }
}

impl<T> IntoIterator for OptTinyVec<T> {
    type Item = T;
    type IntoIter = Map<TinyVecIterator<[Option<T>; LIMIT]>, fn(Option<T>) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().map(|it| it.unwrap())
    }
}
