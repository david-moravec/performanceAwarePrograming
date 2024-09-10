use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Block, ItemFn, Signature, Visibility};

#[proc_macro_attribute]
pub fn time_it(args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as syn::ItemFn);
    let _ = args;

    decorate_fn(attrs, vis, sig, block).into()
}

fn decorate_fn(
    attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    block: Box<Block>,
) -> proc_macro2::TokenStream {
    if &sig.ident.to_string() == "main" {
        decorate_main(attrs, vis, sig, block)
    } else {
        decorate_non_main(attrs, vis, sig, block)
    }
}

fn decorate_non_main(
    attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    block: Box<Block>,
) -> proc_macro2::TokenStream {
    let ident = &sig.ident.to_string();

    quote! {
        #(#attrs)*
        #vis #sig {
            unsafe {
            ::timer_macros::TIMER.start(#ident);
            };

            let ret = #block;

            unsafe {
            ::timer_macros::TIMER.stop(#ident);
            };
            ret
        }
    }
}

fn decorate_main(
    attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    block: Box<Block>,
) -> proc_macro2::TokenStream {
    let ident = &sig.ident.to_string();

    quote! {
        #(#attrs)*
        #vis #sig {
            unsafe {
                ::timer_macros::TIMER.start(#ident);
            };

            let ret = #block;

            unsafe {
                ::timer_macros::TIMER.stop(#ident);
            };
            unsafe {
                ::timer_macros::print_timer();
            };
            ret
        }
    }
}
