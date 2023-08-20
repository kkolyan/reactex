use crate::common;
use crate::common::aggregate_errors;
use crate::common::Argument;
use crate::common::ArgumentType;
use crate::on_signal::ecs_filter_expression;
use crate::on_signal::extract_argument;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use quote::TokenStreamExt;
use std::mem;
use std::str::FromStr;
use syn::fold::fold_stmt;
use syn::fold::Fold;
use syn::parse2;
use syn::spanned::Spanned;
use syn::Block;
use syn::Error;
use syn::Expr;
use syn::ExprClosure;
use syn::Item;
use syn::ItemFn;
use syn::Meta;
use syn::Pat;
use syn::PatType;
use syn::Result;
use syn::Stmt;

pub fn enable_queries(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(attr.span(), "no parameters supported"));
    }
    let item_fn = parse2::<ItemFn>(item.clone())?;

    let mut visitor = MyVisitor {
        registratons: vec![],
        errors: vec![],
        next_wrapper_id: 0,
    };
    let item_fn = visitor.fold_item_fn(item_fn);

    aggregate_errors(visitor.errors.into_iter())?;

    let registration_stmts =
        TokenStream::from_iter(visitor.registratons.iter().map(|it| it.to_token_stream()));

    let register_queries_fn = format_ident!("{}__register_queries", item_fn.sig.ident);

    Ok(quote! {
        #[reactex_core::ctor::ctor]
        fn #register_queries_fn() {
            #registration_stmts
        }
        #item_fn
    })
}

fn transform_closure(
    visitor: &mut MyVisitor,
    mut closure: ExprClosure,
    statements_before: &mut Vec<Stmt>,
) -> Expr {
    let mut query_attr_params = None;
    closure.attrs.retain_mut(|it| match &it.meta {
        Meta::List(it) => {
            let matched = it.path.segments.last().unwrap().ident == "query";
            if matched {
                query_attr_params = Some(it.tokens.clone());
            }
            !matched
        }
        _ => true,
    });

    if query_attr_params.is_none() {
        return Expr::Closure(closure);
    }
    let query_attr_params = query_attr_params.unwrap();
    let query_attr_params_span = query_attr_params.span();
    let ctx = parse2::<Ident>(query_attr_params).map_err(|_| {
        Error::new(
            query_attr_params_span,
            "valid Ctx variable identifier expected as query parameter",
        )
    });
    let ctx = match ctx {
        Ok(it) => it,
        Err(err) => {
            visitor.errors.push(err);
            return Expr::Closure(closure);
        }
    };
    let args = closure.inputs.iter().map(|it| match it {
        Pat::Type(pat_type) => extract_argument(pat_type).map(|it| (pat_type, it)),
        it => Err(Error::new(it.span(), "invalid argument")),
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
                visitor.errors.push(Error::new(
                    *span,
                    "Ctx is not allowed in queries - use one from the containing function",
                ));
                return Expr::Closure(closure);
            }
            ArgumentType::Entity(_) => quote! {
                let #name = __entity__;
            },
            ArgumentType::ComponentReference(ty) => quote! {
                let #name = __entity__.get::<#ty>().unwrap();
            },
            ArgumentType::ComponentMutableWrapper(ty) => quote! {
                let #name = ::reactex_core::facade_2_0::Mut::<#ty>::try_new(__entity__).unwrap();
            },
            ArgumentType::OptionalComponentReference(ty) => quote! {
                let #name = __entity__.get::<#ty>();
            },
            ArgumentType::OptionalComponentMutableWrapper(ty) => quote! {
                let #name = ::reactex_core::facade_2_0::Mut::<#ty>::try_new(__entity__);
            },
        });
    }

    let ecs_filter = ecs_filter_expression(result.iter().map(|(_, arg)| arg));

    visitor.registratons.push(
        parse2::<Stmt>(quote! {
            reactex_core::world_mod::world::register_query(#ecs_filter);
        })
        .unwrap(),
    );

    let arg_decls = TokenStream::from_iter(
        result
            .iter()
            .map(|(pat_type, _)| &pat_type.ty)
            .map(|it| quote!(#it,)),
    );
    let arg_passes = TokenStream::from_iter(
        result
            .iter()
            .map(|(pat_type, _)| &pat_type.pat)
            .map(|it| quote!(#it,)),
    );
    let wrapper_name =
        TokenStream::from_str(format!("__perform_query__{:03}", visitor.next_wrapper_id).as_str())
            .unwrap();
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
    statements_before.push(parse2::<Stmt>(wrapper.clone()).unwrap_or_else(|err| {
        panic!(
            "failed to parse wrapper due to '{}':\n {}",
            err,
            wrapper.to_string()
        )
    }));
    parse2::<Expr>(quote! {
        #wrapper_name(#ctx, #closure)
    })
    .unwrap()
}

struct MyVisitor {
    registratons: Vec<Stmt>,
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
                        Expr::Closure(closure) => transform_closure(self, closure, &mut stmts),
                        expr => expr,
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

pub fn query(_attr: TokenStream, _item: TokenStream) -> Result<TokenStream> {
    Ok(quote! {
        compile_error!("place `#[enable_queries]` at the enclosing top-level function to enable queries");
    })
}
