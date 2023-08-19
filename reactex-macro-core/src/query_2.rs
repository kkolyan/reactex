use std::collections::{HashSet, VecDeque};
use std::mem;
use std::str::FromStr;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Block, Expr, Item, Meta, parse2, Pat, PatType, Stmt};
use syn::spanned::Spanned;
use syn::visit_mut::{visit_expr_closure_mut, visit_expr_mut};
use syn::visit_mut::VisitMut;
use syn::Error;
use syn::ExprClosure;
use syn::fold::{Fold, fold_block, fold_expr, fold_expr_closure, fold_stmt};
use syn::ItemFn;
use syn::Result;
use to_vec::ToVec;
use crate::common;
use crate::common::{aggregate_errors, Argument, ArgumentType};
use crate::on_signal::{ecs_filter_expression, extract_argument};

pub fn enable_queries(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(attr.span(), "no parameters supported"));
    }
    let item_fn = parse2::<ItemFn>(item.clone())?;

    let mut visitor = MyVisitor {
        items: vec![],
        errors: vec![],
        next_wrapper_id: 0,
    };
    let item_fn = visitor.fold_item_fn(item_fn);

    aggregate_errors(visitor.errors.into_iter())?;

    let mut output = TokenStream::new();
    output.append_all(visitor.items.into_iter());

    output.append_all(item_fn.to_token_stream());

    Ok(output)
}

// fn visit_expr_closure_mut(&mut self, item: &mut ExprClosure) {
//     if !item.attrs.is_empty() {
//         let args = item.inputs.iter()
//             .map(|it| match it {
//                 Pat::Type(it) => extract_argument(it),
//                 it => Err(Error::new(it.span(), "invalid argument"))
//             });
//         match common::aggregate_results(args) {
//             Ok(args) => {
//                 let args: Vec<Argument> = args;
//                 quote!{
//
//                     }
//             }
//             Err(err) => {
//                 self.errors.push(err);
//             }
//         };
//         self.errors.push(Error::new(
//             item.span(),
//             format!("visit_expr_closure_mut: {:?}", item),
//         ));
//         item.attrs.clear();
//     }
//     visit_expr_closure_mut(self, item);
//
//     // println!("{:?}", i.to_token_stream());
// }

fn transform_closure(visitor: &mut MyVisitor, mut closure: ExprClosure, statements_before: &mut Vec<Stmt>) -> Expr {
    let mut query_attr_params = None;
    closure.attrs.retain_mut(|it| {
        match &it.meta {
            Meta::List(it) => {
                let matched = it.path.segments.last().unwrap().ident == "query";
                if matched {
                    query_attr_params = Some(it.tokens.clone());
                }
                !matched
            }
            _ => true,
        }
    });

    if query_attr_params.is_none() {
        return Expr::Closure(closure);
    }
    let query_attr_params = query_attr_params.unwrap();
    let query_attr_params_span = query_attr_params.span();
    let ctx = parse2::<Ident>(query_attr_params).map_err(|_| {
        Error::new(query_attr_params_span, "valid Ctx variable identifier expected as query parameter")
    });
    let ctx = match ctx {
        Ok(it) => it,
        Err(err) => {
            visitor.errors.push(err);
            return Expr::Closure(closure);
        }
    };
    let args = closure.inputs.iter()
        .map(|it| match it {
            Pat::Type(pat_type) => extract_argument(pat_type).map(|it| (pat_type, it)),
            it => Err(Error::new(it.span(), "invalid argument"))
        });
    let result: Result<Vec<(&PatType, Argument)>> = common::aggregate_results(args);
    let result = match result {
        Ok(it) => it,
        Err(err) => {
            visitor.errors.push(err);
            return Expr::Closure(closure);
        }
    };

    let mut assignments = TokenStream::new();
    for (_, Argument(name, ty)) in &result {
        assignments.append_all(match ty {
            ArgumentType::Ctx(span, _) => {
                visitor.errors.push(Error::new(*span, "Ctx is not allowed in queries - use one from the containing function"));
                return Expr::Closure(closure);
            },
            ArgumentType::ComponentReference(ty) => quote!{
                let #name = __entity__.get::<#ty>().unwrap();
            },
            ArgumentType::ComponentMutableWrapper(ty) => quote!{
                let #name = ::reactex_core::facade_2_0::Mut::<#ty>::new(__entity__);
            },
            ArgumentType::Entity(_) => quote!{
                let #name = __entity__;
            }
        });
    }

    let ecs_filter = ecs_filter_expression(result.iter().map(|(_, arg)| arg));

    let arg_decls = TokenStream::from_iter(result.iter()
        .map(|(pat_type, _)| &pat_type.ty)
        .map(|it| quote!(#it,)));
    let arg_passes = TokenStream::from_iter(result.iter()
        .map(|(pat_type, _)| &pat_type.pat)
        .map(|it| quote!(#it,)));
    let wrapper_name = TokenStream::from_str(format!("__perform_query__{:03}", visitor.next_wrapper_id).as_str()).unwrap();
    visitor.next_wrapper_id += 1;
    let wrapper = quote! {
        fn #wrapper_name(#ctx: ::reactex_core::facade_2_0::Ctx, mut callback: impl FnMut(#arg_decls)) {
            #ctx.query(#ecs_filter, |entity_key| {
                let __entity__ = #ctx.get_entity(entity_key).unwrap();
                #assignments
                callback(#arg_passes);
            });
        }
    };
    statements_before.push(parse2::<Stmt>(wrapper.clone()).unwrap_or_else(|err| panic!("failed to parse wrapper due to '{}':\n {}", err, wrapper.to_string())));
    parse2::<Expr>(quote! {
        #wrapper_name(#ctx, #closure)
    }).unwrap()
}

struct MyVisitor {
    items: Vec<Item>,
    errors: Vec<Error>,
    next_wrapper_id: u32,
}

impl Fold for MyVisitor {
    fn fold_block(&mut self, mut block: Block) -> Block {
        let mut stmts = Vec::with_capacity(block.stmts.capacity());
        for stmt in mem::take(&mut block.stmts) {
            let stmt = match stmt {
                Stmt::Expr(expr, token) => {
                    let expr = match expr {
                        Expr::Closure(closure) => {
                            transform_closure(self, closure, &mut stmts)
                        }
                        expr => expr
                    };
                    Stmt::Expr(expr, token)
                }
                stmt => stmt,
            };
            let stmt = fold_stmt(self, stmt);
            stmts.push(stmt);
        }
        block.stmts = stmts;
        block
    }
}
