use std::mem;

#[derive(Clone, Debug)]
pub enum VarOptimisticList<T, const LIMIT: usize> {
    Inline([Option<T>; LIMIT], usize),
    Heap(Vec<T>),
}

impl<T, const LIMIT: usize> Default for VarOptimisticList<T, LIMIT> {
    fn default() -> Self {
        VarOptimisticList::Inline(Default::default())
    }
}

impl<T, const LIMIT: usize> VarOptimisticList<T, LIMIT> {
    pub fn push(&mut self, cause: T) {
        let temp = mem::replace(self, VarOptimisticList::None);
        *self = match temp {
            VarOptimisticList::Inline(array, len) => {
                if len >= LIMIT {
                    VarOptimisticList::Heap(Vec::from(&array[..]))
                } else {
                    array[len]
                }
            }
            VarOptimisticList::None => VarOptimisticList::One(cause),
            VarOptimisticList::One(a) => VarOptimisticList::Two(a, cause),
            VarOptimisticList::Two(a, b) => VarOptimisticList::Heap(vec![a, b, cause]),
            VarOptimisticList::Heap(mut list) => {
                list.push(cause);
                VarOptimisticList::Heap(list)
            }
        }
    }

    pub fn iter(&self) -> OptimisticListIter<T> {
        OptimisticListIter { list: self, progress: 0 }
    }
}

pub struct OptimisticListIter<'a, T> {
    list: &'a VarOptimisticList<T>,
    progress: usize,
}

impl<'a, T> Iterator for OptimisticListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self.list {
            VarOptimisticList::None => None,
            VarOptimisticList::One(a) => match self.progress {
                0 => Some(a),
                _ => None,
            }
            VarOptimisticList::Two(a, b) => match self.progress {
                0 => Some(a),
                1 => Some(b),
                _ => None,
            }
            VarOptimisticList::Heap(list) => list.get(self.progress)
        };
        self.progress += 1;
        value
    }
}
