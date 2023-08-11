use std::mem;

#[derive(Clone, Debug)]
pub enum OptimisticList<T> {
    None,
    One(T),
    Two(T, T),
    Multiple(Vec<T>),
}

impl<T> Default for OptimisticList<T> {
    fn default() -> Self {
        OptimisticList::None
    }
}

impl<T> OptimisticList<T> {
    pub fn push(&mut self, cause: T) {
        let temp = mem::replace(self, OptimisticList::None);
        *self = match temp {
            OptimisticList::None => OptimisticList::One(cause),
            OptimisticList::One(a) => OptimisticList::Two(a, cause),
            OptimisticList::Two(a, b) => OptimisticList::Multiple(vec![a, b, cause]),
            OptimisticList::Multiple(mut list) => {
                list.push(cause);
                OptimisticList::Multiple(list)
            }
        }
    }

    pub fn iter(&self) -> OptimisticListIter<T> {
        OptimisticListIter {
            list: self,
            progress: 0,
        }
    }
}

pub struct OptimisticListIter<'a, T> {
    list: &'a OptimisticList<T>,
    progress: usize,
}

impl<'a, T> Iterator for OptimisticListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self.list {
            OptimisticList::None => None,
            OptimisticList::One(a) => match self.progress {
                0 => Some(a),
                _ => None,
            },
            OptimisticList::Two(a, b) => match self.progress {
                0 => Some(a),
                1 => Some(b),
                _ => None,
            },
            OptimisticList::Multiple(list) => list.get(self.progress),
        };
        self.progress += 1;
        value
    }
}
