// #![feature(stmt_expr_attributes)]
// #![feature(proc_macro_hygiene)]
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;

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

pub fn derive_ecs_component(item: TokenStream) -> TokenStream {
    let s: syn::ItemStruct = parse2(item).unwrap();
    let ty = s.ident;
    quote! {
        impl reactex_core::StaticComponentType for #ty {
            // hello
        }
    }
}
