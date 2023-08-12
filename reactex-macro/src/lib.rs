use proc_macro::TokenStream;
use reactex_macro_core::on_signal::EventType;

#[proc_macro]
pub fn query_fn1(input: TokenStream) -> TokenStream {
    reactex_macro_core::query::query_fn1(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn query_fn(input: TokenStream) -> TokenStream {
    reactex_macro_core::query::query_fn(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactex_macro_core::query::query_attr(attr.into(), item.into())
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
    reactex_macro_core::derive_ecs_component(item.into()).into()
}
