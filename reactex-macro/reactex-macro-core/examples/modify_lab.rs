use quote::quote;
use syn::*;
use pretty::pretty_print_expr;
use reactex_macro_core::pretty;

fn main() {
    let result = reactex_macro_core::modify::modify(quote!(ctx, || health.health -= explosion.damage));
    println!("{}", pretty_print_expr(result));
}