use std::collections::HashMap;

struct SomeIterator<TKeys> {
    keys: TKeys,
    values: Vec<i32>,
}

impl<TKeys> SomeIterator<TKeys>
where
    TKeys: Iterator<Item = usize>,
{
    fn next(&mut self) -> Option<&i32> {
        match self.keys.next() {
            None => None,
            Some(it) => self.values.get(it),
        }
    }
}

fn main() {
    let keys: Vec<usize> = vec![2, 4, 7];
    let map = vec![10, 20, 30, 40, 50, 60, 70, 80, 90];
    let mut iterator = SomeIterator {
        keys: keys.into_iter(),
        values: map,
    };

    assert_eq!(iterator.next(), Some(&30));
    assert_eq!(iterator.next(), Some(&50));
    assert_eq!(iterator.next(), Some(&80));
    assert_eq!(iterator.next(), None);
}
