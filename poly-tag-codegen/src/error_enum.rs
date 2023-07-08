use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str, Attribute, Data, DeriveInput, Ident, LitInt, LitStr, Meta, Path,
};

pub fn error_enum(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let input = if let Data::Enum(input) = &derive.data {
        input
    } else {
        return TokenStream::new();
    };
    let (impl_generics, ty_generics, where_clause) = derive.generics.split_for_impl();

    let ty_name = &derive.ident;
    let mut into_status_impls = Vec::new();

    for variant in &input.variants {
        let ident = &variant.ident;
        let status_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("status"));

        match status_attr {
            Some(Attribute {
                meta: Meta::List(meta_list),
                ..
            }) => {
                let item = match meta_list.parse_args::<StatusAttrItem>() {
                    Ok(item) => item,
                    Err(err) => {
                        abort_call_site!(err);
                    }
                };

                match item {
                    StatusAttrItem::FieldName(name) => {
                        let name = match parse_str::<Ident>(&name.value())
                            .map(|name| quote! { #name })
                            .or_else(|_| {
                                parse_str::<LitInt>(&name.value()).map(|name| quote! { #name })
                            }) {
                            Ok(name) => name,
                            Err(_) => {
                                abort_call_site!("invalid field name: {}", name.value());
                            }
                        };
                        into_status_impls.push(quote! {
                            #ty_name::#ident { #name: field, .. } => <_ as crate::response::IntoStatus>::into_status(field),
                        });
                    }
                    StatusAttrItem::Path(path) => {
                        into_status_impls.push(quote! {
                            #ty_name::#ident { .. } => #path,
                        });
                    }
                }
            }
            _ => {
                into_status_impls.push(quote! {
                    #ty_name::#ident { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        }
    }

    TokenStream::from(quote! {
        impl #impl_generics crate::response::IntoStatus for #ty_name #ty_generics #where_clause {
            fn into_status(&self) -> axum::http::StatusCode {
                match self {
                    #(#into_status_impls)*
                }
            }
        }

        impl #impl_generics axum::response::IntoResponse for #ty_name #ty_generics #where_clause {
            fn into_response(self) -> axum::response::Response {
                let status = <_ as crate::response::IntoStatus>::into_status(&self);
                #[cfg(debug_assertions)]
                let body = axum::Json(serde_json::json!({
                    "error": format!("{:#?}", self)
                }));
                #[cfg(not(debug_assertions))]
                let body = axum::Json(serde_json::json!({
                    "error": self.to_string()
                }));
                (status, body).into_response()
            }
        }
    })
}

#[derive(Debug, Clone)]
enum StatusAttrItem {
    FieldName(LitStr),
    Path(Path),
}

impl Parse for StatusAttrItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::FieldName(input.parse()?))
        } else {
            Ok(Self::Path(input.parse()?))
        }
    }
}
