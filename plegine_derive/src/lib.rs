#![crate_type = "proc-macro"]
#![feature(proc_macro_diagnostic)]

mod config;

#[proc_macro_derive(Config)]
pub fn config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    config::config_derive_impl(&ast).into()
}
