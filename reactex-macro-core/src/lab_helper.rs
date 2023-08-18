use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;
use syn::File;

pub fn print_expression(stream: syn::Result<TokenStream>) -> String {
    let stream = stream.unwrap_or_else(syn::Error::into_compile_error);
    let stream = quote! {
        fn main() {
            #stream
        }
    };
    let result = parse2::<File>(stream.clone());
    let result = match result {
        Ok(it) => it,
        Err(err) => {
            panic!("failed to parse:\n{}\ndue to {}", stream, err);
        }
    };
    let wrapped = prettyplease::unparse(&result);
    let lines = wrapped.lines().collect::<Vec<_>>();
    lines[1..lines.len() - 1].join("\n")
}
pub fn print_item(stream: syn::Result<TokenStream>) -> String {
    let stream = stream.unwrap_or_else(syn::Error::into_compile_error);
    prettyplease::unparse(&parse2(stream).unwrap())
}
