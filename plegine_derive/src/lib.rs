#![crate_type = "proc-macro"]

mod config;
mod from_value;

#[proc_macro_derive(Config)]
pub fn config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    config::config_derive_impl(&ast).into()
}

#[proc_macro_derive(FromValue)]
pub fn from_value_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    from_value::from_value_derive_impl(&ast).into()
}
