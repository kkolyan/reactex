fn main() {
    let mut a = A { x: 1, y: 2, z: 3 };
    dodo(&mut a);
    assert_eq!("1,5,10", format!("{},{},{}", a.x, a.y, a.z))
}

fn dodo(a: &mut A) {
    dododo(&a.x, &a.x, &mut a.y, &mut a.z);
}

fn dododo(a: &i32, a1: &i32, b: &mut i32, c: &mut i32) {
    *b = *a + *a1 + *c;
    *c = *b * 2;
}

struct A {
    x: i32,
    y: i32,
    z: i32,
}
