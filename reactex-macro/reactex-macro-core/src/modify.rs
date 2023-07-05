use std::ops::{Deref, Not};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::*;
use syn::spanned::Spanned;
use crate::common::CtxClosureParse;


pub fn modify(input: TokenStream) -> Result<TokenStream> {
    let CtxClosureParse {ctx, closure} = parse2(input.into())?;
    if closure.inputs.is_empty().not() {
        return Err(Error::new(closure.inputs.span(), "no arguments expected"));
    }
    let modification = closure.body.deref();
    let base = match modification {
        Expr::Assign(it) => it.left.deref(),
        Expr::Binary(it) => it.left.deref(),
        Expr::Call(_) => todo!(),
        Expr::MethodCall(it) => it.receiver.deref(),
        Expr::Unary(it) => it.expr.deref(),
        it => {return Err(Error::new(it.span(), "update expression must be one of: assignment, method call, operator"));}
    };
    let base = match base {
        Expr::Field(it) => it.base.deref(),
        _ => return Err(Error::new(base.span(), "left part of update expression should be a field or the identifier of component"))
    };
    Ok(quote!{
        ctx.changes.update_mut_wrapper(&#base, |#base| {
            #modification
        })
    })
}