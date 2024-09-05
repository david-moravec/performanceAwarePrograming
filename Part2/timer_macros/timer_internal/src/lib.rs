use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn time_it(args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as syn::ItemFn);
    let _ = args;

    let ident = &sig.ident.to_string();

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            unsafe {
            ::timer_macros::TIMER.start(#ident.to_string());
            };

            let ret = #block;

            unsafe {
            ::timer_macros::TIMER.stop(&#ident.to_string());
            };
            ret
        }
    };

    expanded.into()
}
