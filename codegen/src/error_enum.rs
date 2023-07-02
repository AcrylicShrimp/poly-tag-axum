use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta, Path};

pub fn error_enum(item: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(item as DeriveInput);
    let input = if let Data::Enum(input) = &derive.data {
        input
    } else {
        return TokenStream::new();
    };
    let has_impl_status = derive
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("impl_status"));

    let ty_name = &derive.ident;
    let mut into_status_impls = Vec::new();

    for variant in &input.variants {
        let ident = &variant.ident;
        let fields = match &variant.fields {
            Fields::Named(_) => {
                quote! { { .. } }
            }
            Fields::Unnamed(_) => {
                quote! { (..) }
            }
            Fields::Unit => {
                quote! {}
            }
        };
        let status_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("status"));

        match status_attr {
            Some(Attribute {
                meta: Meta::List(meta_list),
                ..
            }) => {
                let path = match meta_list.parse_args::<Path>() {
                    Ok(path) => path,
                    Err(err) => {
                        abort_call_site!(err);
                    }
                };
                into_status_impls.push(quote! {
                    #ty_name::#ident #fields => #path,
                });
            }
            _ => {
                into_status_impls.push(quote! {
                    #ty_name::#ident #fields => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                });
            }
        }
    }

    let into_status_impl = if has_impl_status {
        quote! {}
    } else {
        quote! {
            impl crate::response::IntoStatus for #ty_name {
                fn into_status(&self) -> axum::http::StatusCode {
                    match self {
                        #(#into_status_impls)*
                    }
                }
            }
        }
    };

    TokenStream::from(quote! {
        #into_status_impl

        impl axum::response::IntoResponse for #ty_name {
            fn into_response(self) -> axum::response::Response {
                let status_code = <Self as crate::response::IntoStatus>::into_status(&self);
                #[cfg(debug_assertions)]
                let body = axum::Json(serde_json::json!({
                    "error": format!("{:#?}", self)
                }));
                #[cfg(not(debug_assertions))]
                let body = axum::Json(serde_json::json!({
                    "error": self.to_string()
                }));
                (status_code, body).into_response()
            }
        }
    })
}
