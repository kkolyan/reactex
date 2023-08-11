use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;

pub fn print(stream: syn::Result<TokenStream>) -> String {
    let stream = stream.unwrap_or_else(syn::Error::into_compile_error);
    let stream = quote! {
        fn main() {
            #stream
        }
    };
    prettyplease::unparse(&parse2(stream).unwrap())
}
