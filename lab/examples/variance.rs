trait Animal {
    fn say(&self) -> String;
}

struct Cat {}

struct Dog {}

impl Animal for Cat {
    fn say(&self) -> String {
        "myow".to_owned()
    }
}

impl Animal for Dog {
    fn say(&self) -> String {
        "woof".to_owned()
    }
}

fn main() {
    let mut cat = Cat {};
    let mut dog = Dog {};
    let pet_dog: &mut dyn Animal = &mut dog;
    let pet_cat: &mut dyn Animal = &mut cat;
    *pet_cat = *pet_dog;
}
