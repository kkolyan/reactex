use lab_helper::print_expression;
use quote::quote;
use reactex_macro_core::lab_helper;

fn main() {
    let attr = quote! {
        SOME_USER_MODULE
    };
    let item = quote! {
        fn some_user_function(ctx: Ctx<Update>, explosion: &Explosion, exp_pos: Mut<Position>) {
            some_user_code();
        }
    };
    println!("// SOURCE:");
    println!("{}", print_expression(Ok(item.clone())));
    let result = reactex_macro_core::on_signal::on_signal(attr, item);
    println!();
    println!("// RESULT:");
    println!("{}", print_expression(result));
}
