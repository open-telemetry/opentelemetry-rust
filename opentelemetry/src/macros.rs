use proc_macro::TokenStream;
use syn::{Block, ItemFn, LitStr, parse_macro_input};
use syn::__private::ToTokens;

// Default Values
const DEFAULT_METER_PROVIDER_NAME: &'static str = "default_meter_provider";
const DEFAULT_DESCRIPTION: &'static str = "Empty description!";

// Attributes
const METER_PROVIDER_NAME_ATTR_NAME: &'static str = "meter_provider";
const NAME_ATTR_NAME: &'static str = "name";
const DESCRIPTION_ATTR_NAME: &'static str = "description";

// Messages
const ATTR_ERROR_MESSAGE: &'static str = "unsupported attribute for counted macro!";


#[proc_macro_attribute]
pub fn counted(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    let mut name = format!("fn_{}_count", input.sig.ident.to_string());
    let mut meter_provider = DEFAULT_METER_PROVIDER_NAME.to_string();
    let mut description = DEFAULT_DESCRIPTION.to_string();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident(NAME_ATTR_NAME) {
            let value = meta.value()?.parse::<LitStr>()?;
            name = value.value();
            return Ok(());
        } else if meta.path.is_ident(METER_PROVIDER_NAME_ATTR_NAME) {
            let value = meta.value()?.parse::<LitStr>()?;
            meter_provider = value.value();
            return Ok(());
        } else if meta.path.is_ident(DESCRIPTION_ATTR_NAME) {
            let value = meta.value()?.parse::<LitStr>()?;
            description = value.value();
            return Ok(());
        }

        Err(meta.error(ATTR_ERROR_MESSAGE))
    });

    parse_macro_input!(attr with parser);

    let mut counted_code_block = syn::parse_str::<Block>(&format!(r#"
            {{
                static COUNTER: std::sync::LazyLock<opentelemetry::metrics::Counter<u64>> = std::sync::LazyLock::new(|| {{
                let meter = opentelemetry::global::meter("{}");
                meter.u64_counter("{}").with_description("{}").init()
            }});
                COUNTER.add(1, &[]);
            }}
    "#, meter_provider, name, description)).unwrap();

    counted_code_block.stmts.extend_from_slice(&*input.block.stmts);
    input.block.stmts = counted_code_block.stmts;
    TokenStream::from(input.into_token_stream())
}
