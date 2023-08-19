use crate::common;
use crate::common::extract_components;
use crate::common::generate_entity_arg;
use crate::common::render_component_bindings;
use crate::common::Component;
use crate::common::CtxClosureParse;
use crate::common::ExprListParse;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::ops::Deref;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;

pub fn query_fn1(input: TokenStream) -> Result<TokenStream> {
    let args: ExprListParse = parse2(input)?;
    let mut hacked = Punctuated::new();
    hacked.push(parse_quote!(ctx));
    hacked.push(args.exprs.last().unwrap().clone());
    let args = CtxClosureParse::parse_from_list(hacked);

    query_internal(args)
}

pub fn query_fn(input: TokenStream) -> Result<TokenStream> {
    let args: ExprListParse = parse2(input)?;
    let args = CtxClosureParse::parse_from_list(args.exprs);

    query_internal(args)
}

pub fn query_attr(_attr: TokenStream, _item: TokenStream) -> Result<TokenStream> {
    Ok(quote! {
        compile_error!("place `#[enable_queries]` at the enclosing top-level function to enable queries");
    })
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
            block: Block {
                brace_token: Default::default(),
                stmts: vec![Stmt::Expr(expr.clone(), None)],
            },
        },
    };
    let (entity_arg, entity_ident) = generate_entity_arg();
    normalized.inputs.push(Pat::Type(entity_arg));

    render_component_bindings(&ctx, &components, &mut block.block, &entity_ident);

    normalized.body = Box::new(Expr::Block(block));

    let filter_vec = common::generate_filter_vec(&components);

    Ok(quote! {
        let __filter_key__ : reactex::api::FilterDesc = reactex::api::FilterDesc::new(vec![#filter_vec]);
        #ctx.state.query(
            &__filter_key__,
            #normalized,
        );
    })
}
