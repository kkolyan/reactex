use std::ops::Deref;

use quote::ToTokens;
use syn::{AngleBracketedGenericArguments, Block, Error, Expr, ExprClosure, FnArg, GenericArgument, Ident, parse_quote, Pat, PathArguments, PatIdent, PatType, Stmt, Token, Type, TypePath};
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Token};

pub struct ExprListParse {
    pub exprs: Punctuated<Expr, Token!(,)>
}

impl Parse for ExprListParse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ExprListParse {exprs: Punctuated::<Expr, Token!(,)>::parse_separated_nonempty(input)? })
    }
}

pub struct CtxClosureParse {
    pub ctx: Ident,
    pub closure: ExprClosure,
}

impl CtxClosureParse {
    pub fn parse_from_list(args: Punctuated<Expr, Token!(,)>) -> syn::Result<Self> {
        let args: Vec<_> = args.into_iter().collect();
        if args.len() > 2 {
            return Err(Error::new(Span::call_site(), "two arguments expected"));
        }
        let first_tow_args: [_; 2] = args.try_into().map_err(|_| Error::new(Span::call_site(), "two arguments expected"))?;

        let [ctx, closure] = first_tow_args;

        Self::parse(ctx, Stmt::Expr(closure, None))
    }

    pub fn parse(ctx: Expr, closure: Stmt) -> Result<CtxClosureParse, Error> {
        let ctx = if let Expr::Path(ctx) = ctx {
            if ctx.path.segments.len() != 1 {
                Err(1)
            } else {
                let ctx = ctx.path.segments.first().unwrap();
                Ok(ctx.ident.clone())
            }
        } else {
            Err(2)
        };
        let ctx = ctx.map_err(|err| Error::new(
            Span::call_site(),
            format!("first argument should be identifier of a Ctx or SignalCtx parameter. error_code: {}", err),
        ))?;

        let closure = if let Stmt::Expr(Expr::Closure(closure), _) = closure {
            closure
        } else {
            return Err(Error::new(Span::call_site(), "second argument should be a closure"));
        };
        Ok(CtxClosureParse { ctx, closure })
    }
}

pub struct Component {
    pub local_ident: PatIdent,
    pub ty: TypePath,
    pub mutable: bool,
}

pub enum ComponentResult {
    Ok(Component),
    Error { index: usize, msg: &'static str },
}

pub fn extract_components(closure: &ExprClosure) -> Vec<Component> {
    closure.inputs.iter().enumerate()
        .map(|(index, arg)| {
            let result = extract_component(index, arg);
            match result {
                ComponentResult::Ok(it) => it,
                ComponentResult::Error { index, msg } => panic!("closure parameter {} is not a valid component definition: {}", index + 1, msg),
            }
        })
        .collect()
}

fn extract_component(index: usize, arg: &Pat) -> ComponentResult {
    let PatType { pat: identifier, ty, .. } = match arg {
        Pat::Type(it) => it,
        _ => { return ComponentResult::Error { index, msg: "argument should have a type" }; }
    };

    extract_component_2(index, identifier, ty)
}

pub fn extract_component_2(index: usize, identifier: &Pat, ty: &Type) -> ComponentResult {
    let identifier = match identifier.deref() {
        Pat::Ident(identifier) => {
            identifier
        }
        _ => { return ComponentResult::Error { index, msg: "identifier is too complex (a)" }; }
    };
    match ty.deref() {
        Type::Path(ty) => {
            if ty.path.segments.len() != 1 {
                return ComponentResult::Error { index, msg: "if it's not a reference, then it has to be Mut<T>" };
            }
            let ty = ty.path.segments.first().unwrap();
            if ty.ident != "Mut" {
                return ComponentResult::Error { index, msg: "if it's not a reference, then it has to be Mut<T> (b)" };
            }
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &ty.arguments {
                let args: Vec<_> = args.iter().collect();
                if args.len() != 1 {
                    return ComponentResult::Error { index, msg: "Mut<T> has to have exactly one generic parameter" };
                }
                match args.first().unwrap() {
                    GenericArgument::Type(Type::Path(ty)) => ComponentResult::Ok(Component {
                        ty: ty.clone(),
                        local_ident: identifier.clone(),
                        mutable: true,
                    }),
                    _ => ComponentResult::Error { index, msg: "Mut<T> where T is invalid component type" }
                }
            } else {
                ComponentResult::Error { index, msg: "Mut<T> has to be generic" }
            }
        }
        Type::Reference(ty) => {
            if ty.mutability.is_some() {
                return ComponentResult::Error { index, msg: "please use Mut<T> wrapper for mutable component references in signature" };
            }
            let ty = match ty.elem.deref() {
                Type::Path(ty) => ty,
                _ => { return ComponentResult::Error { index, msg: "T is too complex type" }; }
            };

            ComponentResult::Ok(Component {
                local_ident: identifier.clone(),
                ty: ty.clone(),
                mutable: false,
            })
        }
        _ => ComponentResult::Error { index, msg: "argument type should be &T or Mut<T>" }
    }
}

pub fn render_component_bindings(ctx: &Ident, components: &[Component], block: &mut Block, entity: &Ident) {
    for component in components.iter() {
        let identifier = &component.local_ident;
        let component_type = &component.ty;
        let component_not_found = format!("component {} not found - that's bug in ECS framework", component_type.to_token_stream());
        if component.mutable {
            block.stmts.insert(0, parse_quote! {
                let #identifier = #ctx.state.get_component_for_mut::<#component_type>(#entity).expect(#component_not_found);
            });
        } else {
            block.stmts.insert(0, parse_quote! {
                let #identifier = #ctx.state.get_component::<#component_type>(#entity).expect(#component_not_found);
            });
        }
    }
}

pub fn generate_entity_arg() -> (PatType, Ident) {
    let entity_arg: FnArg = parse_quote! { __entity__: reactex::api::Entity };
    let pat_type = match entity_arg {
        FnArg::Receiver(_) => panic!("WTF"),
        FnArg::Typed(pat_type) => pat_type,
    };
    let ident = match pat_type.pat.deref() {
        Pat::Ident(it) => it.ident.clone(),
        _ => panic!("WTF?"),
    };
    (pat_type, ident)
}

pub fn generate_filter_vec(components: &Vec<Component>, registry: &Expr) -> Punctuated<Expr, Comma> {
    let mut filter_vec: Punctuated<Expr, Token![,]> = Punctuated::new();
    for component in components.iter() {
        let component_type = &component.ty;
        filter_vec.push(parse_quote! { #registry.get_component_type::<#component_type>() });
    }
    filter_vec
}
