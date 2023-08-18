use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use std::fs;
use std::io::ErrorKind;
use syn::parse2;

pub fn derive_ecs_component(item: TokenStream, module_path: &str, types_file: &str) -> TokenStream {
    let s: syn::ItemStruct = parse2(item).unwrap();
    let ty = s.ident;

    let ty_str = format!("::{}::{}", module_path, ty);

    let mut lines = match fs::read_to_string(types_file) {
        Ok(s) => s
            .lines()
            .filter(|it| !it.is_empty())
            .map(|it| it.to_owned())
            .collect::<Vec<_>>(),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => vec![],
            err => panic!("unexpected error: {}", err),
        },
    };
    let index = match lines.iter().enumerate().find(|(_, it)| **it == ty_str) {
        None => {
            let index = lines.len();
            lines.push(ty_str.clone());
            fs::write(types_file, lines.join("\n")).unwrap();
            index as u16
        }
        Some((index, _)) => index as u16,
    };
    let register_type_callback = format_ident!("register_type_callback_{}", ty.to_string());
    let register_type = format_ident!("register_type_{}", ty.to_string());
    quote! {
        impl ::reactex_core::component::EcsComponent for #ty {
            const NAME: &'static str = #ty_str;
            const INDEX: u16 = #index + 1;
        }

        #[::reactex_core::ctor::ctor]
        fn #register_type_callback() {
            ::reactex_core::world_mod::world::register_type(#register_type);
        }

        fn #register_type(world: &mut ::reactex_core::world_mod::world::World) {
            world.register_component::<#ty>();
        }
    }
}
