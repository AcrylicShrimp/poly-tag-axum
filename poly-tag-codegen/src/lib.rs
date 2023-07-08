use proc_macro::TokenStream;
use proc_macro_error::*;

mod error_enum;

#[proc_macro_derive(ErrorEnum, attributes(status))]
#[proc_macro_error]
pub fn error_enum(item: TokenStream) -> TokenStream {
    error_enum::error_enum(item)
}
