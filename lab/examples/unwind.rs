use std::panic::catch_unwind;
use std::sync::Mutex;

fn main() {
    let visited_depth = Mutex::new(0);
    catch_unwind(|| {
        dive(0, &visited_depth);
    });
}

fn dive(depth: usize, found: &Mutex<usize>) {
    if depth > 10 {
        panic!("bottom reached");
    }
    dive(depth + 1, found);
}
