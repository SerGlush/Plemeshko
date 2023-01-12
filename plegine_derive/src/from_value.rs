use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataStruct, Fields, FieldsNamed, FieldsUnnamed};

pub fn from_value_derive_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let from_value_body = match &ast.data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(fields) => from_value_derive_impl_struct_named(name, fields),
            Fields::Unnamed(fields) => from_value_derive_impl_struct_unnamed(name, fields),
            Fields::Unit => quote! { plegine::json::Null::from_value(src)?; Ok(#name) },
        },
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => quote! { compile_error!("Can't derive FromValue for unions.") },
    };

    let gen = quote! {
        impl FromValue for #name {
            fn from_value(src: plegine::json::Value) -> plegine::json::ParseResult<Self> {
                #from_value_body
            }
        }
    };
    gen.into()
}

fn from_value_derive_impl_struct_named(
    con: &Ident,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let src = quote! { &mut src };
    let fields_ts = fields
        .named
        .iter()
        .fold(TokenStream::new(), |mut ts, field| {
            let field_ty = &field.ty;
            let field_id = &field.ident;
            ts.extend(quote! {
                #field_id: <#field_ty as FromValue>::from_value(plegine::json::try_take_key(#src, stringify!(#field_id)).map_err(|e| e.lift(stringify!(#field_id)))?)?,
            });
            ts
        });
    quote! {
        match src {
            plegine::json::Value::Object(mut src) => {
                Ok(#con {
                    #fields_ts
                })
            },
            _ => plegine::json::parse_type_err_res(),
        }
    }
}

fn from_value_derive_impl_struct_unnamed(
    con: &Ident,
    fields: &FieldsUnnamed,
) -> proc_macro2::TokenStream {
    match fields.unnamed.len() {
        0 => quote! { compile_error!("Can't derive FromValue for 0-element tuples.") },
        1 => {
            let field = fields.unnamed.first().unwrap();
            let field_ty = &field.ty;
            quote! {
                Ok(#con(<#field_ty as FromValue>::from_value(src)?))
            }
        }
        _ => {
            let (fields_ts, _) = fields.unnamed.iter().fold(
                (TokenStream::new(), 0usize),
                |(mut ts, index), field| {
                    let field_ty = &field.ty;
                    ts.extend(quote! {
                        <#field_ty as FromValue>::from_value(src[#index].take())?,
                    });
                    (ts, index + 1)
                },
            );
            let fields_len = fields.unnamed.len();
            quote! {
                match src {
                    plegine::json::Value::Array(mut src) => {
                        if src.len() != #fields_len {
                            return plegine::json::parse_type_err_res();
                        }
                        Ok(#con (#fields_ts))
                    },
                    _ => plegine::json::parse_type_err_res(),
                }
            }
        }
    }
}
