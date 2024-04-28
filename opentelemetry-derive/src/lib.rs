use darling::{util::Ignored, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};

#[derive(FromDeriveInput)]
#[darling(attributes(otel))]
struct Options<Key, Variant> {
    key: Key,
    variant: Variant,
}

#[proc_macro_derive(Key, attributes(otel))]
pub fn key(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive(input, key_impl)
}

fn key_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = Options::<Option<String>, Option<Ignored>>::from_derive_input(&input)?;
    let ident = input.ident;
    let key = options.key.unwrap_or(ident.to_string().to_lowercase());
    Ok(quote! {
        impl #ident {
            const KEY: opentelemetry::Key = opentelemetry::Key::from_static_str(#key);
        }

        #[automatically_derived]
        impl From<&#ident> for opentelemetry::Key {
            fn from(_: &#ident) -> Self {
                #ident::KEY
            }
        }

        #[automatically_derived]
        impl From<#ident> for opentelemetry::Key {
            fn from(_: #ident) -> Self {
                #ident::KEY
            }
        }
    })
}

#[proc_macro_derive(KeyValue)]
pub fn key_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive(input, key_value_impl)
}

fn key_value_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident;
    Ok(quote! {
        #[automatically_derived]
        impl From<&#ident> for opentelemetry::KeyValue {
            fn from(value: &#ident) -> Self {
                Self::new(value, value)
            }
        }

        #[automatically_derived]
        impl From<#ident> for opentelemetry::KeyValue {
            fn from(value: #ident) -> Self {
                Self::from(&value)
            }
        }
    })
}

#[proc_macro_derive(StringValue)]
pub fn string_value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive(input, string_value_impl)
}

fn string_value_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident;
    Ok(quote! {
        const _: () = {
            struct AssertToString where #ident: ToString;
        };

        #[automatically_derived]
        impl From<&#ident> for opentelemetry::StringValue {
            fn from(value: &#ident) -> Self {
                value.to_string().into()
            }
        }

        #[automatically_derived]
        impl From<#ident> for opentelemetry::StringValue {
            fn from(value: #ident) -> Self {
                Self::from(&value)
            }
        }
    })
}

#[proc_macro_derive(Value, attributes(otel))]
pub fn value(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive(input, value_impl)
}

fn value_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = Options::<Option<Ignored>, syn::Path>::from_derive_input(&input)?;
    let ident = input.ident;
    let variant = options.variant;
    Ok(quote! {
        #[automatically_derived]
        impl From<&#ident> for opentelemetry::Value {
            fn from(value: &#ident) -> Self {
                #variant::from(value).into()
            }
        }

        #[automatically_derived]
        impl From<#ident> for opentelemetry::Value {
            fn from(value: #ident) -> Self {
                Self::from(&value)
            }
        }
    })
}

fn derive(
    input: proc_macro::TokenStream,
    f: fn(DeriveInput) -> syn::Result<TokenStream>,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    f(input).unwrap_or_else(Error::into_compile_error).into()
}
