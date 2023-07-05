use quote::quote;
use syn::{File, parse2};
use pretty::pretty_print_expr;
use reactex_macro_core::pretty;

fn main() {
    let attr = quote!{
        SOME_USER_MODULE
    };
    let item = quote!{
        fn some_user_function(ctx: Ctx<Update>, explosion: &Explosion, exp_pos: Mut<Position>) {
            some_user_code();
        }
    };
    let result = reactex_macro_core::on_signal::on_signal(attr, item);
    println!("{}", pretty_print_expr(result));
}