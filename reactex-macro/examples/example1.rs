use std::ops::Deref;
use syn::{ExprClosure, FnArg, ItemFn, parse_quote, Pat, Type};

fn main() {

    let fn_ast: ItemFn = parse_quote! {
        fn update_explosion(ctx: SignalCtx<Update>, explosion: &Explosion, exp_pos: Mut<Position>) {}
    };

    let cl_ast: ExprClosure = parse_quote! {
        |ctx: SignalCtx<Update>, explosion: &Explosion, exp_pos: Mut<Position>| {}
    };

    let fn_inputs: Vec<_> = fn_ast.sig.inputs.iter()
        .map(|it| match it {
            FnArg::Receiver(_) => panic!("receiver!"),
            FnArg::Typed(it) => it,
        })
        .collect();
    let cl_inputs: Vec<_> = cl_ast.inputs.iter()
        .map(|it| match it {
            Pat::Type(it) => it,
            _ => panic!(),
        })
        .collect();
    println!();
}