use std::collections::HashMap;

struct A;

impl A {
    fn add_handler<F>(&mut self, callback: F)
        where F: FnMut(HashMap<String, i32>)
    {
        todo!()
    }
    fn add_handler_2<F>(&mut self, callback: F)
        where F: AutoAddableHandler
    {
        callback.add_myself(self);
    }
}

trait AutoAddableHandler {
    fn add_myself(self, s: &mut A);
}

impl <F: Fn(i32)> AutoAddableHandler for F {
    fn add_myself(self, s: &mut A) {
        s.add_handler(|map| {
            self(*map.get("a").unwrap());
        });
    }
}

impl <F: Fn(i32, i32)> AutoAddableHandler for F {
    fn add_myself(self, s: &mut A) {
        s.add_handler(|map| {
            self(*map.get("a").unwrap());
        });
    }
}

// impl HandlerAdder<> for Target

// impl MyInto<>

fn main() {
    A.add_handler(|events| {
        let a = events.get("a").unwrap();
        println!("a: {}", a);
    });
    A.add_handler_2(|a: i32| {
        println!("a: {}", a);
    });
}