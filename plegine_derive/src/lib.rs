#![crate_type = "proc-macro"]

use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{DataStruct, FieldsNamed, Fields};

#[proc_macro_derive(Config)]
pub fn config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    config_derive_impl(&ast).into()
}

fn config_derive_impl_struct_named(src: TokenStream, con: &Ident, fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let fields_ts = fields.named.iter().fold(TokenStream::new(), |mut ts, field| {
        let ident = &field.ident;
        ts.extend(quote!{
            #ident : plegine::json::try_take_key(#src, stringify!(#ident))?,
        });
        ts
    });
    quote!{
        Ok(#con {
            #fields_ts
        })
    }
}

fn config_derive_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let src = quote!{ &mut src };

    let parse_body = match &ast.data {
        syn::Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => config_derive_impl_struct_named(src, name, fields),
        syn::Data::Struct(DataStruct { fields: _, .. }) => todo!(),
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => quote! { compile_error!("Can't derive Config for unions.") },
    };

    let gen = quote! {
        impl Config for #name {
            const TAG: &'static str = stringify!(#name);

            fn parse(mut src: plegine::json::Object) -> Result<Self, plegine::json::ParseError> {
                #parse_body
            }
        }
    };
    gen.into()
}
