use crate::common;
use crate::common::extract_component_2;
use crate::common::render_component_bindings;
use crate::common::Component;
use crate::common::ComponentResult;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::*;
use std::ops::Deref;
use std::ops::Not;
use syn::punctuated::*;
use syn::spanned::Spanned;
use syn::*;

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
    let (module_path, function, ctx, signal_type, components, ctx_ident) =
        extract_input(attr, item, event_type)?;

    let normalized_function =
        generate_normalized_function(&function, (ctx, ctx_ident), &components, event_type);

    let registration =
        generate_registration(module_path, &function, signal_type, &components, event_type);

    Ok(quote! {
        #registration
        #normalized_function
    })
}

fn extract_input(
    attr: TokenStream,
    item: TokenStream,
    event_type: EventType,
) -> Result<(
    ExprPath,
    ItemFn,
    PatType,
    Option<Type>,
    Vec<Component>,
    Ident,
)> {
    let (attribute_name, ctx_type_name) = match event_type {
        EventType::OnSignal => ("on_signal", "Ctx<T>"),
        EventType::OnSignalGlobal => ("on_signal_global", "Ctx<T>"),
        EventType::OnAppear => ("on_appear", "Ctx"),
        EventType::OnDisappear => ("on_disappear", "Ctx"),
    };
    let module_path = parse2::<ExprPath>(attr)?;
    let function = parse2::<ItemFn>(item)?;
    let args: Vec<_> = function
        .sig
        .inputs
        .iter()
        .map(|it| match it {
            FnArg::Receiver(_) => panic!(
                "{} is applicable only to top-level free functions",
                attribute_name
            ),
            FnArg::Typed(it) => it,
        })
        .collect();
    if args.is_empty() {
        panic!(
            "{} requires first argument - {}",
            attribute_name, ctx_type_name
        );
    }
    let ctx = *args.first().unwrap();

    let signal_type =
        extract_signal_type_and_validate_ctx(ctx, event_type).map_err(|ctx_type_error| {
            Error::new(
                ctx.span(),
                format!(
                    "{} first parameter should be {} value (error_code: {})",
                    attribute_name, ctx_type_name, ctx_type_error
                ),
            )
        })?;

    let components = &args[1..];
    if let EventType::OnSignalGlobal = event_type {
        if components.is_empty().not() {
            return Err(Error::new(
                Span::call_site(),
                format!("{} doesn't accept components", attribute_name),
            ));
        }
    }
    let components: Vec<_> = components
        .iter()
        .enumerate()
        .map(|(index, component)| {
            let result = extract_component_2(index, component.pat.deref(), component.ty.deref());
            match result {
                ComponentResult::Ok(it) => it,
                ComponentResult::Error { index, msg } => panic!(
                    "function parameter {} is not a valid component definition: {}",
                    index + 1,
                    msg
                ),
            }
        })
        .collect();

    let ctx_ident = match ctx.pat.deref() {
        Pat::Ident(it) => &it.ident,
        _ => panic!("invalid context variable"),
    };
    Ok((
        module_path,
        function.clone(),
        ctx.clone(),
        signal_type,
        components,
        ctx_ident.clone(),
    ))
}

fn extract_signal_type_and_validate_ctx(
    ctx: &PatType,
    event_type: EventType,
) -> std::result::Result<Option<Type>, &'static str> {
    let signal_type = match ctx.ty.deref() {
        Type::Path(it) => it,
        _ => {
            return Err("invalid ctx type");
        }
    };

    let ident = signal_type.path.segments.last().unwrap();
    if ident.ident != "Ctx" {
        return Err("invalid ctx type name");
    }
    let args = match &ident.arguments {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => args.clone(),
        PathArguments::None => Punctuated::new(),
        _ => {
            return Err("invalid ctx generic type");
        }
    };
    let expected_parameters_count = match event_type {
        EventType::OnSignal | EventType::OnSignalGlobal => 1,
        EventType::OnAppear | EventType::OnDisappear => 0,
    };
    if args.len() != expected_parameters_count {
        return Err("invalid ctx generic parameters count");
    }
    match event_type {
        EventType::OnSignal | EventType::OnSignalGlobal => match args.first().unwrap() {
            GenericArgument::Type(it) => Ok(Some(it.clone())),
            _ => Err("invalid ctx generic parameter"),
        },
        EventType::OnAppear | EventType::OnDisappear => Ok(None),
    }
}

fn generate_normalized_function(
    function: &ItemFn,
    (ctx, ctx_ident): (PatType, Ident),
    components: &Vec<Component>,
    event_type: EventType,
) -> ItemFn {
    let mut normalized_function = function.clone();
    normalized_function.sig.inputs = Punctuated::new();
    normalized_function
        .sig
        .inputs
        .push(FnArg::Typed(ctx.clone()));

    if let EventType::OnSignalGlobal = event_type {
        return normalized_function;
    }

    let (entity_arg, entity_ident) = common::generate_entity_arg();
    normalized_function
        .sig
        .inputs
        .push(FnArg::Typed(entity_arg));

    render_component_bindings(
        &ctx_ident,
        &components,
        &mut normalized_function.block,
        &entity_ident,
    );
    normalized_function
}

fn generate_registration(
    module_path: ExprPath,
    function: &ItemFn,
    signal_type: Option<Type>,
    components: &Vec<Component>,
    event_type: EventType,
) -> ItemFn {
    let filter_key = match event_type {
        EventType::OnSignal | EventType::OnAppear | EventType::OnDisappear => {
            let filter_vec = common::generate_filter_vec(components);
            Some(quote! { let filter_key = reactex::api::FilterKey::new(vec![#filter_vec]); })
        }
        EventType::OnSignalGlobal => None,
    };
    let registration_function_name = format_ident!("register_{}", function.sig.ident);
    let function_name = &function.sig.ident;

    let registration = match event_type {
        EventType::OnSignal => {
            quote! { registry.add_entity_signal_handler::<#signal_type>(filter_key, #function_name); }
        }
        EventType::OnSignalGlobal => {
            quote! { registry.add_global_signal_handler::<#signal_type>(#function_name); }
        }
        EventType::OnAppear => {
            quote! { registry.add_entity_appear_handler(filter_key, #function_name); }
        }
        EventType::OnDisappear => {
            quote! { registry.add_entity_disappear_handler(filter_key, #function_name); }
        }
    };
    parse_quote! {
        #[ctor::ctor]
        fn #registration_function_name() {
            #module_path.write().unwrap().register_later(move || {
                #filter_key
                #registration
            });
        }
    }
}
