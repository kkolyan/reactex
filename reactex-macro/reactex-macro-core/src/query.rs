use std::ops::Deref;
use quote::{quote};
use proc_macro2::TokenStream;
use syn::*;
use syn::punctuated::Punctuated;
use crate::common;
use crate::common::{Component, extract_components, generate_entity_arg, render_component_bindings, CtxClosureParse, ExprListParse};

pub fn query_fn1(input: TokenStream) -> Result<TokenStream> {
    let args : ExprListParse = parse2(input)?;
    let mut hacked = Punctuated::new();
    hacked.push(parse_quote!(ctx));
    hacked.push(args.exprs.last().unwrap().clone());
    let args = CtxClosureParse::parse_from_list(hacked);

    query_internal(args)
}

pub fn query_fn(input: TokenStream) -> Result<TokenStream> {
    let args : ExprListParse = parse2(input)?;
    let args = CtxClosureParse::parse_from_list(args.exprs);

    query_internal(args)
}

pub fn query_attr(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr = parse2(attr);
    let item: Result<Stmt> = parse2(item);
    let args = CtxClosureParse::parse(
        attr?,
        item?,
    );

    query_internal(args)
}

fn query_internal(args: Result<CtxClosureParse>) -> Result<TokenStream> {
    let CtxClosureParse { ctx, closure } = args?;
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

    let filter_vec = common::generate_filter_vec(&components, &parse_quote!(#ctx.state));

    Ok(quote! {
        let __filter_key__ : reactex::api::FilterKey = reactex::api::FilterKey::new(vec![#filter_vec]);
        #ctx.state.query(
            &__filter_key__,
            #normalized,
        );
    })
}