use proc_macro::TokenStream;
use std::ops::Deref;
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, ExprMacro, ExprPath, FnArg, Ident, ItemFn, Local, Macro, MacroDelimiter, parse_quote, Pat, Path, PatType, Stmt, Token, Type};
use syn::__private::Span;
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Token};

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
        .map(|it| { match it {
            FnArg::Receiver(_) => panic!("on_signal is applicable only to top-level free functions"),
            FnArg::Typed(it) => it
        } })
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

    for &component in components {
        let component_pat = component.pat.deref();
        let ctx_pat = &ctx.pat;
        let component_type = match component.ty.deref() {
            Type::Reference(it) => it.elem.deref(),
            _ => panic!("component-holding parameters should be references")
        };
        let component_not_found = format!("component {} not found - that's bug in ECS framework", component_type.to_token_stream());
        normalized.block.stmts.insert(0, parse_quote! {
            let #component_pat = #ctx_pat.state.get_component::<#component_type>(#ctx_pat.entity).expect(#component_not_found);
        });
    }

    let registration_function_name = format_ident!("register_{}", function.sig.ident);

    let mut filter_vec: Punctuated<Expr, Token![,]> = Punctuated::new();
    for &component in components.iter() {
        let component_type = match component.ty.deref() {
            Type::Reference(it) => it.elem.deref(),
            _ => panic!("component-holding parameters should be references")
        };
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