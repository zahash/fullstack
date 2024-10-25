use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use server::Username;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn username(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let pat = lit_str.value();

    if let Err(err) = Username::from_str(&pat) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote! { #pat }.into()
}
