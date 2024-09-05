use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::parse_macro_input;

use util::counter::read_os_timer;

struct TimeElapsed {
    start: u64,
    stop: Option<u64>,
}

impl TimeElapsed {
    pub fn new() -> Self {
        TimeElapsed {
            start: read_os_timer(),
            stop: None,
        }
    }

    pub fn stop(&mut self) -> () {
        self.stop = Some(read_os_timer());
    }
}

struct Timer {
    profiles: HashMap<syn::Ident, Vec<TimeElapsed>>,
}

impl Timer {
    pub fn start(&mut self, ident: syn::Ident) -> () {
        match self.profiles.get_mut(&ident) {
            Some(profile) => profile.push(TimeElapsed::new()),
            None => {
                self.profiles.insert(ident, vec![TimeElapsed::new()]);
            }
        }
    }

    pub fn stop(&mut self, ident: syn::Ident) -> () {
        self.profiles
            .get_mut(&ident)
            .expect("Should always start before stopping")
            .last_mut()
            .expect("Should always start before stopping")
            .stop()
    }
}

#[proc_macro_attribute]
pub fn timer(args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as syn::ItemFn);
    let _ = args;

    eprintln!("{:#?}", parsed);

    quote! {parsed}.into()
}
