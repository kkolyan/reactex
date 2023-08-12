#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

use proc_macro2::TokenStream;
use quote::quote;
use std::fs;
use std::io::ErrorKind;
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

pub fn derive_ecs_component(item: TokenStream, module_path: &str, types_file: &str) -> TokenStream {
    let s: syn::ItemStruct = parse2(item).unwrap();
    let ty = s.ident;

    let ty_str = format!("{}::{}", module_path, ty);

    let mut lines = match fs::read_to_string(types_file) {
        Ok(s) => s
            .lines()
            .filter(|it| !it.is_empty())
            .map(|it| it.to_owned())
            .collect::<Vec<_>>(),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => vec![],
            err => panic!("unexpected error: {}", err),
        },
    };
    let index = match lines.iter().enumerate().find(|(_, it)| **it == ty_str) {
        None => {
            let index = lines.len();
            lines.push(ty_str.clone());
            fs::write(types_file, lines.join("\n")).unwrap();
            index as u16
        }
        Some((index, _)) => index as u16,
    };
    quote! {
        impl ::reactex_core::StaticComponentType for #ty {
            const NAME: &'static str = #ty_str;
            const INDEX: u16 = #index + 1;
        }
    }
}
