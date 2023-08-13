use crate::filter::filter::Filter;
use crate::filter::filter_manager::InternalFilterKey;
use crate::typed_index_vec::{TiVec, TiVecKey};

impl TiVecKey for InternalFilterKey {
    fn from_index(index: usize) -> Self {
        InternalFilterKey(index)
    }
    fn as_index(&self) -> usize {
        self.0
    }
}

pub(crate) struct FilterIter<'a, TKeys> {
    pub(crate) source: &'a mut TiVec<InternalFilterKey, Filter>,
    pub(crate) keys: TKeys,
}

impl<'a, I> FilterIter<'a, I>
where
    I: Iterator<Item = InternalFilterKey>,
{
    pub fn next(&mut self) -> Option<&mut Filter> {
        match self.keys.next() {
            None => None,
            Some(it) => self.source.get_mut(&it),
        }
    }
}
