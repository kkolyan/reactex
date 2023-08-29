pub(crate) struct TemporaryEntityKeyStorage {
    pub(crate) used_holes: usize,
    pub(crate) tail_allocations: usize,
}

impl TemporaryEntityKeyStorage {
    pub(crate) fn new() -> TemporaryEntityKeyStorage {
        TemporaryEntityKeyStorage {
            used_holes: 0,
            tail_allocations: 0,
        }
    }
}
