#[cfg(feature = "regex")]
#[proc_macro]
pub fn regex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let lit_str = syn::parse_macro_input!(input as syn::LitStr);
    let pat = lit_str.value();

    if let Err(err) = regex::Regex::new(&pat) {
        return syn::Error::new(lit_str.span(), err)
            .to_compile_error()
            .into();
    }

    quote::quote! { ::regex::Regex::new(#pat).unwrap() }.into()
}
