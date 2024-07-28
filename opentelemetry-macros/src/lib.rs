use proc_macro::TokenStream;

use syn::{ItemFn, parse_macro_input};

use crate::metrics::counted::CountedBuilder;


#[cfg(feature = "metrics")]
mod metrics;

#[proc_macro_attribute]
#[cfg(feature = "metrics")]
pub fn counted(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut builder = match CountedBuilder::from(attr, parse_macro_input!(item as ItemFn)) {
        Ok(value) => { value }
        Err(err) => {
            return err;
        }
    };

    builder.process()
}
