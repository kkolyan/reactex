use proc_macro2::TokenStream;
use reactex_macro_core::lab_helper::print_item;
use std::str::FromStr;

fn main() {
    let item = TokenStream::from_str(
        "
        struct Y {
            value: i32,
        }
    ",
    )
    .unwrap();
    let result = reactex_macro_core::components::derive_ecs_component(
        item,
        "source/file.rs",
        ".derive_ecs_component.examples.txt",
    );
    println!("{}", result);
    println!("{}", print_item(Ok(result)));
}
