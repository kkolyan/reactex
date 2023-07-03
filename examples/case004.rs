use std::collections::HashMap;

struct A;

impl A {
    fn add_handler(&mut self, callback: Box<dyn FnMut(HashMap<String,i32>)>) {
        todo!()
    }
}

trait Normalizable0 {
    fn normalize(self) -> Box<dyn FnMut(HashMap<String, i32>)>;
}
impl <F> Normalizable0 for F where F: FnMut(HashMap<String,i32>) + 'static
{
    fn normalize(mut self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        Box::from(move |map: HashMap<String, i32>| {
            self(map)
        })
    }
}

trait Normalizable1 {
    fn normalize(self) -> Box<dyn FnMut(HashMap<String, i32>)>;
}
impl <F> Normalizable1 for F where F: FnMut(i32) + 'static
{
    fn normalize(mut self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        Box::from(move |map: HashMap<String, i32>| {
            let a = map.get("a").unwrap();
            self(*a)
        })
    }
}

trait Normalizable2 {
    fn normalize(self) -> Box<dyn FnMut(HashMap<String, i32>)>;
}
impl <F> Normalizable2 for F where F: FnMut(i32, i32) + 'static
{
    fn normalize(mut self) -> Box<dyn FnMut(HashMap<String, i32>)> {
        Box::from(move |map: HashMap<String, i32>| {
            let a = map.get("a").unwrap();
            let b = map.get("b").unwrap();
            self(*a, *b)
        })
    }
}

fn do_std_verbose(map: HashMap<String, i32>) {
    todo!()
}

fn do_1(a: i32) {
    todo!()
}

fn do_2(a: i32, b: i32) {
    todo!()
}

fn main() {
    A.add_handler(do_std_verbose.normalize());
    A.add_handler(do_1.normalize());
    A.add_handler(do_2.normalize());
}