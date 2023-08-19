use lab_helper::print_expression;
use quote::quote;
use reactex_macro_core::lab_helper;

fn main() {
    let attr = quote! {};
    let item = quote! {
        fn some_user_function(ctx: Ctx<Update>, explosion: &Explosion, exp_pos: Mut<Position>) {

            #[query(ctx)]
            |a: &A| {

            };

            #[query(ctx)]
            |a: &A, b: &B, en: Entity| {

            };
        }
    };
    println!("// SOURCE:");
    println!("{}", print_expression(Ok(item.clone())));
    let result = reactex_macro_core::query_2::enable_queries(attr, item);
    println!();
    println!("// RESULT:");
    println!("{}", print_expression(result));
}
