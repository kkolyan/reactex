use proc_macro::TokenStream;

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    reactex_macro_core::query::query(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn modify(input: TokenStream) -> TokenStream {
    reactex_macro_core::modify::modify(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
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
    reactex_macro_core::on_signal::on_signal(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
