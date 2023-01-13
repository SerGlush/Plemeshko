use proc_macro2::TokenStream;
use quote::quote;

pub fn config_derive_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let gen = quote! {
        impl plegine::config::Config for #name {
            const TAG: &'static str = stringify!(#name);
        }
    };
    gen.into()
}
