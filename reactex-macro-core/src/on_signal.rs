use crate::common;
use crate::common::Argument;
use crate::common::ArgumentType;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::*;
use std::ops::Deref;
use syn::punctuated::*;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::*;
use to_vec::ToVec;

#[derive(Copy, Clone)]
pub enum EventType {
    OnSignal,
    OnSignalGlobal,
    OnAppear,
    OnDisappear,
}

pub fn on_signal(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    on_event(attr, item, EventType::OnSignal)
}

pub fn on_event(
    attr: TokenStream,
    item: TokenStream,
    event_type: EventType,
) -> Result<TokenStream> {
    let user_function = analyze_user_function(attr, item.clone())?;

    let registration = generate_registration_new(user_function, event_type)?;

    Ok(quote! {
        #registration
        #item
    })
}

struct UserFunction {
    ecs_module_var_path: ExprPath,
    args: Vec<Argument>,
    ident: Ident,
    args_span: Span,
}

fn analyze_user_function(attr: TokenStream, item: TokenStream) -> Result<UserFunction> {
    let ecs_module_var_path = parse2::<ExprPath>(attr)?;

    let function = parse2::<ItemFn>(item)
        .map_err(|err| Error::new(err.span(), "attribute is applicable only to functions"))?;

    let args_span = function.sig.span();
    let ident = function.sig.ident;

    let args = function.sig.inputs.iter().map(|it| match it {
        FnArg::Receiver(it) => Err(Error::new(
            it.span(),
            "attribute is applicable only to top-level free functions",
        )),
        FnArg::Typed(arg) => extract_argument(arg),
    });
    let args = common::aggregate_results(args)?;

    Ok(UserFunction {
        args_span,
        ident,
        ecs_module_var_path,
        args,
    })
}

pub(crate) fn extract_argument(arg: &PatType) -> Result<Argument> {
    let arg_name = match arg.pat.deref() {
        Pat::Ident(it) => Pat::Ident(it.clone()),
        Pat::Wild(it) => Pat::Wild(it.clone()),
        it => return Err(Error::new(it.span(), "invalid argument name")),
    };
    let result = match arg.ty.deref() {
        Type::Path(it) => {
            let it = it.path.segments.last().unwrap();
            if it.ident == "Ctx" {
                let payload = extract_single_generic_argument_type(it)?;
                Ok(Argument(arg_name, ArgumentType::Ctx(it.span(), payload)))
            } else if it.ident == "Mut" {
                let component = extract_single_generic_argument_type(it)?;
                match component {
                    None => Err(Error::new(
                        it.span(),
                        "exactly one generic argument expected here",
                    )),
                    Some(it) => Ok(Argument(
                        arg_name,
                        ArgumentType::ComponentMutableWrapper(it),
                    )),
                }
            } else if it.ident == "Entity" {
                Ok(Argument(arg_name, ArgumentType::Entity(it.span())))
            } else {
                Err(Error::new(it.span(), "invalid argument type"))
            }
        }
        Type::Reference(it) => {
            if it.lifetime.is_some() {
                Err(Error::new(
                    it.span(),
                    "explicit lifetimes are not expected here",
                ))
            } else if it.mutability.is_some() {
                Err(Error::new(
                    it.span(),
                    "mutability is not allowed here. us Mut wrapper instead.",
                ))
            } else {
                Ok(Argument(
                    arg_name,
                    ArgumentType::ComponentReference(it.elem.deref().clone()),
                ))
            }
        }
        it => Err(Error::new(it.span(), "invalid argument type")),
    };
    result
}

fn extract_single_generic_argument_type(it: &PathSegment) -> Result<Option<Type>> {
    match &it.arguments {
        PathArguments::None => Ok(None),
        PathArguments::AngleBracketed(it) => {
            let params = it.args.iter().to_vec();
            if params.len() == 1 {
                match params.first().unwrap() {
                    GenericArgument::Type(it) => Ok(Some(it.clone())),
                    it => Err(Error::new(
                        it.span(),
                        "generic argument could be only a type",
                    )),
                }
            } else {
                Err(Error::new(
                    it.span(),
                    "at most 1 generic argument expected here",
                ))
            }
        }
        PathArguments::Parenthesized(it) => Err(Error::new(
            it.span(),
            "parenthesized arguments are not expected here",
        )),
    }
}

fn generate_registration_new(
    user_function: UserFunction,
    event_type: EventType,
) -> Result<TokenStream> {
    let requests_entity = user_function.args.iter().any(|Argument(_, ty)| match ty {
        ArgumentType::Entity(_) => true,
        ArgumentType::Ctx(_, _)
        | ArgumentType::ComponentReference(_)
        | ArgumentType::ComponentMutableWrapper(_) => false,
    });
    match event_type {
        EventType::OnSignalGlobal => {
            if requests_entity {
                return Err(Error::new(
                    user_function.args_span,
                    "no Entity is associated with this event",
                ));
            }
        }
        EventType::OnSignal | EventType::OnAppear | EventType::OnDisappear => {}
    }

    let filter_key = match event_type {
        EventType::OnSignal | EventType::OnAppear | EventType::OnDisappear => {
            ecs_filter_expression(user_function.args.iter())
        }
        EventType::OnSignalGlobal => None,
    };
    let function_name = &user_function.ident;
    let registration_function_name = format_ident!("register_{}", function_name);

    let function_args = TokenStream::from_iter(
        user_function
            .args
            .iter()
            .map(|Argument(name, ty)| quote! {#name,}),
    );

    let argument_mappings = TokenStream::from_iter(user_function.args.iter().map(
        |Argument(name, ty)| match ty {
            ArgumentType::Ctx(_, _) => quote! {
                let #name = __ctx__;
            },
            ArgumentType::ComponentReference(ty) => quote! {
                let #name = __entity__.get::<#ty>().unwrap();
            },
            ArgumentType::ComponentMutableWrapper(ty) => quote! {
                let #name = ::reactex_core::facade_2_0::Mut::<#ty>::new(__entity__);
            },
            ArgumentType::Entity(_) => quote! {
                let #name = __entity__;
            },
        },
    ));

    match event_type {
        EventType::OnSignalGlobal => {
            let mut component_types = user_function
                .args
                .iter()
                .filter_map(|Argument(_, ty)| match ty {
                    ArgumentType::Ctx(_, _) => None,
                    ArgumentType::Entity(span) => Some(*span),
                    ArgumentType::ComponentReference(it) => Some(it.span()),
                    ArgumentType::ComponentMutableWrapper(it) => Some(it.span()),
                })
                .map(|it| {
                    Error::new(
                        it,
                        "global events cannot be associated with any entities and components",
                    )
                });

            let error = component_types.next();
            if let Some(mut error) = error {
                error.extend(component_types);
                return Err(error);
            }
        }
        EventType::OnSignal | EventType::OnAppear | EventType::OnDisappear => {
            let entity_or_component_args_present =
                user_function.args.iter().any(|Argument(_, ty)| match ty {
                    ArgumentType::Ctx(_, _) => false,
                    ArgumentType::ComponentReference(_)
                    | ArgumentType::Entity(_)
                    | ArgumentType::ComponentMutableWrapper(_) => true,
                });
            if !entity_or_component_args_present {
                return Err(Error::new(
                    user_function.args_span,
                    "it doesn't make sense without entity or component arguments, so prohibited",
                ));
            }
        }
    };

    let signal_type = user_function
        .args
        .iter()
        .filter_map(|Argument(_, ty)| match ty {
            ArgumentType::Ctx(span, it) => Some((span, it)),
            _ => None,
        })
        .to_vec();

    if signal_type.len() > 1 {
        let mut errors = signal_type
            .into_iter()
            .map(|(span, _)| Error::new(*span, "at most 1 Ctx argument expected"));
        let mut error = errors.next().unwrap();
        error.extend(errors);
        return Err(error);
    }
    let signal_type = signal_type.into_iter().next();

    let signal_type = match event_type {
        EventType::OnSignal | EventType::OnSignalGlobal => {
            let signal_type = match signal_type {
                None => {
                    return Err(Error::new(
                        user_function.args_span,
                        "exactly 1 Ctx argument expected for this event",
                    ));
                }
                Some((span, signal_type)) => match signal_type {
                    None => return Err(Error::new(
                        *span,
                        "Ctx<..> is expected to be parameterized with payload type for this event",
                    )),
                    Some(signal_type) => match signal_type {
                        Type::Path(signal_type) => signal_type.clone(),
                        it => {
                            return Err(Error::new(
                                it.span(),
                                "invalid payload type for this event",
                            ));
                        }
                    },
                },
            };
            Some(signal_type)
        }
        EventType::OnAppear | EventType::OnDisappear => {
            if let Some((span, signal_type)) = signal_type {
                if signal_type.is_some() {
                    return Err(Error::new(
                        *span,
                        "payload type not expected for this event",
                    ));
                }
            }
            None
        }
    };

    let registration = match event_type {
        EventType::OnSignal => {
            quote! {
                fn wrapper(signal: &#signal_type, entity: reactex_core::entity::EntityKey, stable: &reactex_core::world_mod::world::StableWorld, volatile: &mut reactex_core::world_mod::world::VolatileWorld) {
                    let volatile = std::cell::RefCell::new(volatile);
                    let __ctx__ = Ctx::new(signal, stable, &volatile);
                    let __entity__ = __ctx__.get_entity(entity).unwrap();
                    #argument_mappings
                    #function_name(#function_args);
                }
                world.add_entity_signal_handler::<#signal_type>(stringify!(#function_name), #filter_key, wrapper);
            }
        }
        EventType::OnSignalGlobal => {
            quote! {
                fn wrapper(signal: &#signal_type, stable: &reactex_core::world_mod::world::StableWorld,volatile: &mut reactex_core::world_mod::world::VolatileWorld) {
                    let volatile = std::cell::RefCell::new(volatile);
                    let __ctx__ = Ctx::new(signal, stable, &volatile);
                    #argument_mappings
                    #function_name(#function_args);
                }
                world.add_global_signal_handler::<#signal_type>(stringify!(#function_name), wrapper);
            }
        }
        EventType::OnAppear => {
            quote! {
                fn wrapper(entity: reactex_core::entity::EntityKey, stable: &reactex_core::world_mod::world::StableWorld, volatile: &mut reactex_core::world_mod::world::VolatileWorld) {
                    let volatile = std::cell::RefCell::new(volatile);
                    let __ctx__ = Ctx::new(&(), stable, &volatile);
                    let __entity__ = __ctx__.get_entity(entity).unwrap();
                    #argument_mappings
                    #function_name(#function_args);
                }
                world.add_appear_handler(stringify!(#function_name), #filter_key, wrapper);
            }
        }
        EventType::OnDisappear => {
            quote! {
                fn wrapper(entity: reactex_core::entity::EntityKey, stable: &reactex_core::world_mod::world::StableWorld, volatile: &mut reactex_core::world_mod::world::VolatileWorld) {
                    let volatile = std::cell::RefCell::new(volatile);
                    let __ctx__ = Ctx::new(&(), stable, &volatile);
                    let __entity__ = __ctx__.get_entity(entity).unwrap();
                    #argument_mappings
                    #function_name(#function_args);
                }
                world.add_disappear_handler(stringify!(#function_name), #filter_key, wrapper);
            }
        }
    };
    let ecs_module_path = user_function.ecs_module_var_path;
    Ok(quote! {
        #[::reactex_core::ctor::ctor]
        fn #registration_function_name() {
            fn configure(world: &mut ::reactex_core::world_mod::world::ConfigurableWorld) {
                #registration
            }
            #ecs_module_path.write().unwrap().add_configurator(configure);
        }
    })
}

pub(crate) fn ecs_filter_expression<'a>(
    iter: impl Iterator<Item = &'a Argument>,
) -> Option<TokenStream> {
    let components = iter.filter_map(|Argument(_, ty)| match ty {
        ArgumentType::Ctx(_, _) => None,
        ArgumentType::ComponentReference(it) => Some(it),
        ArgumentType::Entity(_) => None,
        ArgumentType::ComponentMutableWrapper(it) => Some(it),
    });
    let components: Punctuated<&Type, Comma> = Punctuated::from_iter(components);
    Some(quote! {
        ::reactex_core::filter::filter_desc::ecs_filter!(#components)
    })
}
