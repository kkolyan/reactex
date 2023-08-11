// #![feature(stmt_expr_attributes)]
// #![feature(proc_macro_hygiene)]
use proc_macro2::TokenStream;

pub mod common;
pub mod lab_helper;
pub mod on_signal;
pub mod query;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn on_signal_global(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn on_appear(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn on_disappear(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
