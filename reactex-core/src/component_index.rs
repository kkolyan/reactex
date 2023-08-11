use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

static NEXT_COMPONENT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn next_component_index() -> usize {
    loop {
        let index = NEXT_COMPONENT_INDEX.load(SeqCst);
        if NEXT_COMPONENT_INDEX
            .compare_exchange(index, index + 1, SeqCst, SeqCst)
            .is_ok()
        {
            return index;
        }
    }
}
