use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;

pub fn print_expression(stream: syn::Result<TokenStream>) -> String {
    let stream = stream.unwrap_or_else(syn::Error::into_compile_error);
    let stream = quote! {
        fn main() {
            #stream
        }
    };
    prettyplease::unparse(&parse2(stream).unwrap())
}
pub fn print_item(stream: syn::Result<TokenStream>) -> String {
    let stream = stream.unwrap_or_else(syn::Error::into_compile_error);
    prettyplease::unparse(&parse2(stream).unwrap())
}
