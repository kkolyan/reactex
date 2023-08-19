// #![feature(proc_macro_span)]
#![feature(proc_macro_expand)]

use proc_macro::TokenStream;
use reactex_macro_core::on_signal::EventType;
use std::str::FromStr;
use syn::parse;

#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::query::query(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn enable_queries(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::query::enable_queries(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn on_signal_global(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::on_signal::on_event(attr.into(), item.into(), EventType::OnSignalGlobal)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn on_appear(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::on_signal::on_event(attr.into(), item.into(), EventType::OnAppear)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn on_disappear(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::on_signal::on_event(attr.into(), item.into(), EventType::OnDisappear)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn on_signal(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::on_signal::on_event(attr.into(), item.into(), EventType::OnSignal)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(EcsComponent)]
pub fn derive_ecs_component(item: TokenStream) -> TokenStream {
    let module_path = resolve_module_path();

    let file = match module_path {
        Err(_) => ".derive_ecs_component.ide.txt",
        Ok(_) => ".derive_ecs_component.macro.txt",
    };
    let module_path = match module_path {
        Ok(it) => it,
        Err(it) => it,
    };
    reactex_macro_core::components::derive_ecs_component(item.into(), module_path.as_str(), file)
        .into()
}

fn resolve_module_path() -> Result<String, String> {
    let module_path_macro_call =
        TokenStream::from_str("module_path!()").map_err(|err| err.to_string())?;
    let module_path_literal = module_path_macro_call
        .expand_expr()
        .map_err(|err| err.to_string())?;
    let module_path_literal_parsed =
        parse::<syn::LitStr>(module_path_literal.clone()).map_err(|err| {
            format!(
                "err: {:?}, source: {}",
                err.to_string(),
                module_path_literal
            )
        })?;
    Ok(module_path_literal_parsed.value())
}
