use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use server::{Email, Password, Username};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn username(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let value = lit_str.value();

    if let Err(err) = Username::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote! { #value }.into()
}

#[proc_macro]
pub fn password(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let value = lit_str.value();

    if let Err(err) = Password::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote! { #value }.into()
}

#[proc_macro]
pub fn email(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let value = lit_str.value();

    if let Err(err) = Email::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote! { #value }.into()
}
