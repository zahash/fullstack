#[cfg(feature = "username")]
#[proc_macro]
pub fn username(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use std::str::FromStr;

    let lit_str = syn::parse_macro_input!(input as syn::LitStr);
    let value = lit_str.value();

    if let Err(err) = server::Username::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote::quote! { #value }.into()
}

#[cfg(feature = "password")]
#[proc_macro]
pub fn password(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use std::str::FromStr;

    let lit_str = syn::parse_macro_input!(input as syn::LitStr);
    let value = lit_str.value();

    if let Err(err) = server::Password::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote::quote! { #value }.into()
}

#[cfg(feature = "email")]
#[proc_macro]
pub fn email(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use std::str::FromStr;

    let lit_str = syn::parse_macro_input!(input as syn::LitStr);
    let value = lit_str.value();

    if let Err(err) = server::Email::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote::quote! { #value }.into()
}
