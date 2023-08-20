pub fn boxed_slice<T: Clone>(template: T, capacity: usize) -> Box<[T]> {
    vec![template; capacity].into_boxed_slice()
}
