#[cfg(feature = "username")]
#[proc_macro]
pub fn username(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    domain_type_compiletime_check::<server_core::Username>(input)
}

#[cfg(feature = "password")]
#[proc_macro]
pub fn password(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    domain_type_compiletime_check::<server_core::Password>(input)
}

#[cfg(feature = "email")]
#[proc_macro]
pub fn email(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    domain_type_compiletime_check::<server_core::Email>(input)
}

fn domain_type_compiletime_check<T>(input: proc_macro::TokenStream) -> proc_macro::TokenStream
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let lit_str = syn::parse_macro_input!(input as syn::LitStr);
    let value = lit_str.value();

    if let Err(err) = T::from_str(&value) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote::quote! { #value }.into()
}
