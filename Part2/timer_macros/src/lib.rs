use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn timer(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as syn::ItemFn);
    let _ = args;

    eprintln!("{:#?}", parsed);

    quote! {parsed}.into()
}
