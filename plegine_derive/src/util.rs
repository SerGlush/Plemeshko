use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Lit, LitStr};

pub fn json_optional_key_w_default(src: &TokenStream, field: &Field) -> TokenStream {
    let field_ty = &field.ty;
    let field_id = &field.ident;
    let mut attr_optional = false;
    let mut attr_default: Option<LitStr> = None;
    for attr in field.attrs.iter() {
        let Ok(meta) = attr.parse_meta() else {
            proc_macro::Span::error(proc_macro::Span::call_site(), "Failed to parse attribute").emit();
            return quote!{};
        };
        match meta {
            syn::Meta::Path(p) => {
                if p.is_ident("default") {
                    attr_optional = true;
                }
            }
            syn::Meta::List(ml) => {
                if ml.path.is_ident("default") {
                    proc_macro::Span::error(ml.path.segments.first().unwrap().ident.span().unwrap(), "`default` attribute should be of the form \"#[default]\" or \"#[default = value]\".").emit();
                }
            }
            syn::Meta::NameValue(mnv) => {
                if mnv.path.is_ident("default") {
                    match mnv.lit {
                        Lit::Str(expr) => attr_default = Some(expr),
                        _ => proc_macro::Span::error(
                            mnv.lit.span().unwrap(),
                            "Only expressions in string literals are allowed as default values.",
                        )
                        .emit(),
                    }
                }
            }
        }
    }
    let default = if attr_optional {
        if attr_default.is_some() {
            return quote! { compile_error!("`optional` and `default` attributes are mutually exclusive.") };
        }
        quote! { <#field_ty as Default>::default() }
    } else if let Some(value) = attr_default {
        value
            .parse::<TokenStream>()
            .unwrap_or_else(|e| e.to_compile_error())
    } else {
        quote! { return Err(plegine::json::ParseError::new_absent(stringify!(#field_id).to_string())) }
    };
    quote! {
        #field_id: match plegine::json::try_take_optional_key::<#field_ty>(#src, stringify!(#field_id))? {
            Some(value) => value,
            None => #default,
        },
    }
}
