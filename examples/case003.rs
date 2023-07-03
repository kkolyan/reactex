use std::collections::HashMap;

struct A;

trait Registry {
    fn add_handler<F>(&mut self, callback: F)
        where F: FnMut(HashMap<String, i32>);
}

impl Registry for A {
    fn add_handler<F>(&mut self, callback: F)
        where F: FnMut(HashMap<String, i32>) {
        todo!()
    }
}

trait SugaredRegistry1 : Registry {
    fn add_handler<F>(&mut self, callback: F)
    where F: FnMut(i32) {
        Registry::add_handler(self, |map| {
            let a = map.get("a").unwrap();
            callback(*a);
        });
    }
}

impl <T: Registry> SugaredRegistry1 for T {}

trait SugaredRegistry2 : Registry {
    fn add_handler<F>(&mut self, callback: F)
    where F: FnMut(i32, i32) {
        Registry::add_handler(self, |map| {
            let a = map.get("a").unwrap();
            let b = map.get("a").unwrap();
            callback(*a, *b);
        });
    }
}

fn main() {
    A.add_handler(|events: HashMap<String, i32>| {
        let a = events.get("a").unwrap();
        println!("a: {}", a);
    });
    A.add_handler(|a: i32| {
        println!("a: {}", a);
    });
    A.add_handler(|a: i32, b: i32| {
        println!("a: {}, b: {}", a, b);
    });
}