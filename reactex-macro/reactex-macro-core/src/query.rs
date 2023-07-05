use std::ops::Deref;
use quote::{quote};
use proc_macro2::TokenStream;
use syn::*;
use crate::common;
use crate::common::{Component, extract_components, generate_entity_arg, render_component_bindings, CtxClosureParse};

pub fn query(input: TokenStream) -> Result<TokenStream> {
    let CtxClosureParse { ctx, closure } = parse2(input)?;
    let components: Vec<Component> = extract_components(&closure);
    let mut normalized = closure;
    normalized.inputs.clear();

    let mut block = match normalized.body.deref() {
        Expr::Block(block) => block.clone(),
        expr => ExprBlock {
            attrs: vec![],
            label: None,
            block: Block { brace_token: Default::default(), stmts: vec![Stmt::Expr(expr.clone(), None)] },
        }
    };
    let (entity_arg, entity_ident) = generate_entity_arg();
    normalized.inputs.push(Pat::Type(entity_arg));

    render_component_bindings(&ctx, &components, &mut block.block, &entity_ident);

    normalized.body = Box::new(Expr::Block(block));

    let filter_vec = common::generate_filter_vec(components, &parse_quote!(#ctx.state));

    Ok(quote!{
        let __filter_key__ : reactex::api::FilterKey = reactex::api::FilterKey::new(vec![#filter_vec]);
        #ctx.state.query(
            &__filter_key__,
            #normalized,
        );
    })
}