use proc_macro::TokenStream;
use std::ops::Deref;

use quote::{format_ident, ToTokens};
use syn::{AngleBracketedGenericArguments, Block, Error, Expr, ExprBlock, ExprClosure, ExprPath, FnArg, GenericArgument, ItemFn, parse_macro_input, parse_quote, Pat, PathArguments, PatIdent, PatType, Stmt, Token, Type, TypePath};
use syn::__private::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

struct SubQueryInputParse {
    ctx: ExprPath,
    closure: ExprClosure,
}

impl Parse for SubQueryInputParse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Expr, Token!(,)>::parse_terminated(input)?;
        let args: Vec<_> = args.into_iter().collect();
        if args.len() > 2 {
            return Err(Error::new(Span::call_site(), "two arguments expected"));
        }
        let first_tow_args: [_; 2] = args.try_into().map_err(|_| Error::new(Span::call_site(), "two arguments expected"))?;

        let [ctx, closure] = first_tow_args;
        let ctx = if let Expr::Path(ctx) = ctx {
            ctx
        } else {
            return Err(Error::new(Span::call_site(), "first argument should be a reference to an EntityCtx or GlobalCtx"));
        };

        let closure = if let Expr::Closure(closure) = closure {
            closure
        } else {
            return Err(Error::new(Span::call_site(), "second argument should be a closure"));
        };
        Ok(SubQueryInputParse { ctx, closure })
    }
}

struct Component {
    local_ident: PatIdent,
    ty: TypePath,
}

enum ComponentResult {
    Ok(Component),
    Error { index: usize, msg: &'static str },
}


#[proc_macro]
pub fn sub_query(input: TokenStream) -> TokenStream {
    let SubQueryInputParse { ctx, closure } = parse_macro_input!(input);
    let args: Vec<_> = closure.inputs.iter().collect();
    // panic!("debug: {:?}", closure.inputs.to_token_stream());
    let components: Vec<Component> = extract_components(&closure);
    let mut normalized = closure.clone();
    normalized.inputs.clear();

    let mut block = match normalized.body.deref() {
        Expr::Block(block) => block.clone(),
        expr => ExprBlock {
            attrs: vec![],
            label: None,
            block: Block { brace_token: Default::default(), stmts: vec![Stmt::Expr(expr.clone(), None)] },
        }
    };
    for component in components {
        let ident = component.local_ident;
        let component_type = component.ty;
        let component_not_found = format!("component {} not found - that's bug in ECS framework", component_type.to_token_stream());
        block.block.stmts.insert(0, parse_quote! {
            let #ident = #ctx.state.get_component::<#component_type>(#ctx.entity).expect(#component_not_found);
        });
    }
    normalized.body = Box::new(Expr::Block(block));
    normalized.into_token_stream().into()
}

fn extract_components(closure: &ExprClosure) -> Vec<Component> {
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

fn extract_component_2(index: usize, identifier: &Pat, ty: &Type) -> ComponentResult {
    let identifier = match identifier.deref() {
        Pat::Ident(identifier) => {
            identifier
        }
        _ => { return (ComponentResult::Error { index, msg: "identifier is too complex (a)" }); }
    };
    match ty.deref() {
        Type::Path(ty) => {
            if ty.path.segments.len() != 1 {
                return (ComponentResult::Error { index, msg: "if it's not a reference, then it has to be Mut<T>" });
            }
            let ty = ty.path.segments.first().unwrap();
            if ty.ident != "Mut" {
                return (ComponentResult::Error { index, msg: "if it's not a reference, then it has to be Mut<T> (b)" });
            }
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &ty.arguments {
                let args: Vec<_> = args.iter().collect();
                if args.len() != 1 {
                    return (ComponentResult::Error { index, msg: "Mut<T> has to have exactly one generic parameter" });
                }
                match args.first().unwrap() {
                    GenericArgument::Type(Type::Path(ty)) => ComponentResult::Ok(Component { ty: ty.clone(), local_ident: identifier.clone() }),
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
                _ => { return ComponentResult::Error { index, msg: "Mut<T> where T is too complex type" }; }
            };

            ty.path.segments.last()
                .map(|it| ComponentResult::Ok(Component {
                    local_ident: identifier.clone(),
                    ty: ty.clone(),
                }))
                .unwrap_or_else(|| ComponentResult::Error { index, msg: "Mut<T> where T is invalid component type" })
        }
        _ => ComponentResult::Error { index, msg: "argument type should be &T or Mut<T>" }
    }
}

#[proc_macro_attribute]
pub fn on_signal_global(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_appear(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_disappear(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn on_signal(attr: TokenStream, item: TokenStream) -> TokenStream {
    let module_path = syn::parse::<ExprPath>(attr).expect("on_signal should refer to a RwLock<Module> static variable");
    let function = syn::parse::<ItemFn>(item).expect("on_signal is applicable only for free-standing functions");
    let args: Vec<_> = function.sig.inputs.iter()
        .map(|it| {
            match it {
                FnArg::Receiver(_) => panic!("on_signal is applicable only to top-level free functions"),
                FnArg::Typed(it) => it
            }
        })
        .collect();
    if args.len() < 2 {
        panic!("on_signal requires at least two arguments - first is signal payload ref, last is EntityCtx value");
    }
    let signal = *args.first().unwrap();
    let ctx = *args.last().unwrap();
    let components = &args[1..args.len() - 1];

    match ctx.ty.deref() {
        Type::Path(it) if it.path.segments.last().unwrap().ident == "EntityCtx" => (),
        _ => {
            panic!("on_signal last parameter should be EntityCtx value")
        }
    };

    let mut normalized = function.clone();
    normalized.sig.inputs = Punctuated::new();
    normalized.sig.inputs.push(FnArg::Typed(signal.clone()));
    normalized.sig.inputs.push(FnArg::Typed(ctx.clone()));

    let components : Vec<_> = components.iter().enumerate()
        .map(|(index, component)| {
            let result = extract_component_2(index, component.pat.deref(), component.ty.deref());
            match result {
                ComponentResult::Ok(it) => it,
                ComponentResult::Error { index, msg } => panic!("function parameter {} is not a valid component definition: {}", index + 1, msg),
            }
        })
        .collect();

    for component in components.iter() {
        let identifier = &component.local_ident;
        let component_type = &component.ty;
        let ctx_pat = &ctx.pat;
        let component_not_found = format!("component {} not found - that's bug in ECS framework", component_type.to_token_stream());
        normalized.block.stmts.insert(0, parse_quote! {
            let #identifier = #ctx_pat.state.get_component::<#component_type>(#ctx_pat.entity).expect(#component_not_found);
        });
    }

    let registration_function_name = format_ident!("register_{}", function.sig.ident);

    let mut filter_vec: Punctuated<Expr, Token![,]> = Punctuated::new();
    for component in components.iter() {
        let component_type = &component.ty;
        filter_vec.push(parse_quote! { registry.get_component_type::<#component_type>() });
    }

    let signal_type = match signal.ty.deref() {
        Type::Reference(it) => it.elem.deref(),
        _ => panic!("signal payload holding parameters should be references")
    };
    let registration: ItemFn = parse_quote! {
        #[ctor::ctor]
        fn #registration_function_name() {
            #module_path.write().unwrap().register_later(move |registry| {
                let filter_key = reactex::api::FilterKey::new(vec![#filter_vec]);
                registry.add_entity_signal_handler::<#signal_type>(filter_key, update_explosion);
            });
        }
    };

    let total: syn::File = parse_quote! {
        #registration
        #normalized
    };
    total.to_token_stream().into()
}