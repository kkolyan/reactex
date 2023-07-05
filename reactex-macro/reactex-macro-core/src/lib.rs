#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]
use proc_macro2::TokenStream;

pub mod common;
pub mod on_signal;
pub mod query;
pub mod lab_helper;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub fn on_signal_global(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn on_appear(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn on_disappear(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}