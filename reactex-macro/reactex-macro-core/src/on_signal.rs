use std::ops::Deref;
use proc_macro2::TokenStream;
use quote::*;
use syn::*;
use syn::punctuated::*;
use syn::spanned::Spanned;
use crate::common;
use crate::common::{ComponentResult, extract_component_2, render_component_bindings};

pub fn on_signal(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let module_path = parse2::<ExprPath>(attr)?;
    let function = parse2::<ItemFn>(item)?;
    let args: Vec<_> = function.sig.inputs.iter()
        .map(|it| {
            match it {
                FnArg::Receiver(_) => panic!("on_signal is applicable only to top-level free functions"),
                FnArg::Typed(it) => it
            }
        })
        .collect();
    if args.is_empty() {
        panic!("on_signal requires first argument - Ctx<T>");
    }
    let ctx = *args.first().unwrap();
    let components = &args[1..];

    let signal_type = match ctx.ty.deref() {
        Type::Path(it) => {
            let ident = it.path.segments.last().unwrap();
            if ident.ident == "Ctx" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &ident.arguments {
                    if args.len() == 1 {
                        match args.first().unwrap() {
                            GenericArgument::Type(it) => Ok(it),
                            _ => Err(1)
                        }
                    } else {
                        Err(2)
                    }
                } else {
                    Err(3)
                }
            } else {
                Err(4)
            }
        }
        _ => Err(5)
    };
    let signal_type = signal_type.unwrap_or_else(|ctx_type_error| {
        panic!("on_signal first parameter should be Ctx<T> value (error_code: {})", ctx_type_error)
    });

    let mut normalized = function.clone();
    normalized.sig.inputs = Punctuated::new();
    normalized.sig.inputs.push(FnArg::Typed(ctx.clone()));

    let components: Vec<_> = components.iter().enumerate()
        .map(|(index, component)| {
            let result = extract_component_2(index, component.pat.deref(), component.ty.deref());
            match result {
                ComponentResult::Ok(it) => it,
                ComponentResult::Error { index, msg } => panic!("function parameter {} is not a valid component definition: {}", index + 1, msg),
            }
        })
        .collect();

    let ctx = match ctx.pat.deref() {
        Pat::Ident(it) => &it.ident,
        _ => panic!("invalid context variable")
    };

    let (entity_arg, entity_ident) = common::generate_entity_arg();
    normalized.sig.inputs.push(FnArg::Typed(entity_arg));

    render_component_bindings(ctx, &components, &mut normalized.block, &entity_ident);

    let registration_function_name = format_ident!("register_{}", function.sig.ident);

    let registry = parse_quote! {registry};

    let filter_vec = common::generate_filter_vec(components, &registry);

    let registration: ItemFn = parse_quote! {
        #[ctor::ctor]
        fn #registration_function_name() {
            #module_path.write().unwrap().register_later(move |#registry| {
                let filter_key = reactex::api::FilterKey::new(vec![#filter_vec]);
                registry.add_entity_signal_handler::<#signal_type>(filter_key, update_explosion);
            });
        }
    };

    let total: File = parse_quote! {
        #registration
        #normalized
    };
    Ok(total.to_token_stream())
}