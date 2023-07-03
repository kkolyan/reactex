use std::collections::HashMap;

struct A;

impl A {
    fn add_handler<F>(&mut self, callback: F)
        where F: LambdaBoxer
    {
        todo!()
    }
}

trait LambdaBoxer {
    fn to_box(self) -> Box<dyn FnMut(HashMap<String, i32>)>;
}

impl <F: FnMut(HashMap<String, i32>) + 'static> LambdaBoxer for F {
    fn to_box(self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        Box::from(self)
    }
}

trait LambdaBoxer1 {
    fn to_box_1(self) -> Box<dyn FnMut(HashMap<String, i32>)>;
}

impl <F: FnMut(i32) + 'static> LambdaBoxer1 for F {
    fn to_box_1(mut self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        Box::from(move |map: HashMap<String, i32>| {
            self(*map.get("a").unwrap())
        })
    }
}

trait LambdaBoxer2 {
    fn to_box_2(self) -> Box<dyn FnMut(HashMap<String, i32>)>;
}

impl <F: FnMut(i32, i32) + 'static> LambdaBoxer2 for F {
    fn to_box_2(mut self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        Box::from(move |map: HashMap<String, i32>| {
            self(*map.get("a").unwrap(), *map.get("b").unwrap())
        })
    }
}

impl <T: LambdaBoxer1> LambdaBoxer for T {
    fn to_box(self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        self.to_box_1()
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