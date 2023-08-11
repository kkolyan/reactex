use std::any::TypeId;
use std::mem::size_of;

fn main() {
    println!("{}", size_of::<TypeId>())
}
