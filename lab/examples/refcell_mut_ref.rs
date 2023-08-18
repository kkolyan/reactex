use std::cell::RefCell;

struct A {
    a: i32,
}

fn main() {
    let mut a = A { a: 0 };
    let a0: RefCell<&mut A> = RefCell::new(&mut a);
    let a1 = &a0;
    let a2 = &a0;

    dodo(a1, a2);
    println!("a: {}", a.a);
}

fn dodo(a1: &RefCell<&mut A>, a2: &RefCell<&mut A>) {
    a1.borrow_mut().a = 12;
    let i = a1.borrow().a * 2;
    a2.borrow_mut().a = i;
}
