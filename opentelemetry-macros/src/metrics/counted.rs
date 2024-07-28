use proc_macro::TokenStream;
use std::sync::Mutex;

use syn::{Block, ItemFn, LitStr};
use syn::__private::ToTokens;
use syn::parse::Parser;

pub struct CountedBuilder {
    name: String,
    description: String,
    meter_provider: String,
    labels: String,
    item_fn: Option<ItemFn>,
}

impl CountedBuilder {
    pub fn from(attr: TokenStream, item_fn: ItemFn) -> Result<CountedBuilder, TokenStream> {
        const DEFAULT_METER_PROVIDER_NAME: &'static str = "default_meter_provider";
        const DEFAULT_DESCRIPTION: &'static str = "Empty description!";
        const METER_PROVIDER_NAME_ATTR_NAME: &'static str = "meter_provider";
        const NAME_ATTR_NAME: &'static str = "name";
        const DESCRIPTION_ATTR_NAME: &'static str = "description";
        const LABELS_ATTR_NAME: &'static str = "labels";
        const ATTR_ERROR_MESSAGE: &'static str = "unsupported attribute for counted macro!";

        let mut name = format!("fn_{}_count", item_fn.sig.ident.to_string());
        let mut meter_provider = DEFAULT_METER_PROVIDER_NAME.to_string();
        let mut description = DEFAULT_DESCRIPTION.to_string();
        let mut labels: Vec<String> = vec![];
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
            } else if meta.path.is_ident(LABELS_ATTR_NAME) {
                let value = meta.value()?.parse::<LitStr>()?;
                let raw_labels = value.value();
                labels = raw_labels.split(",").into_iter().map(|v| v.trim().to_string()).collect();
                if labels.len() % 2 != 0 {
                    panic!("invalid arguments provided in labels attribute! (must be provided list of key-value)");
                }
                return Ok(());
            }

            Err(meta.error(ATTR_ERROR_MESSAGE))
        });

        if let Err(err) = parser.parse(attr) {
            return Err(TokenStream::from(err.to_compile_error()));
        }

        let mut labels_as_str = String::new();
        for chunk in labels.chunks(2) {
            if let [key, value] = chunk {
                if !labels_as_str.is_empty() {
                    labels_as_str.push_str(", ");
                }
                labels_as_str.push_str(&format!(r#"KeyValue::new("{}", "{}")"#, key, value));
            }
        }

        Ok(Self {
            name,
            description,
            meter_provider,
            labels: labels_as_str,
            item_fn: Some(item_fn),
        })
    }
    fn check_metric_name(&self) {
        static METRIC_NAMES: std::sync::LazyLock<Mutex<Vec<String>>> = std::sync::LazyLock::new(|| {
            Mutex::new(Vec::new())
        });

        let mut vec_lock = METRIC_NAMES.lock().unwrap();
        if vec_lock.contains(&self.name.as_str().to_string()) {
            panic!("detected metric name duplication!");
        }
        vec_lock.push(self.name.clone())
    }
    fn build_code_block(&self) -> Block {
        syn::parse_str::<Block>(&format!(r#"
            {{
                static COUNTER: std::sync::LazyLock<opentelemetry::metrics::Counter<u64>> = std::sync::LazyLock::new(|| {{
                let meter = opentelemetry::global::meter("{}");
                meter.u64_counter("{}").with_description("{}").init()
            }});
                COUNTER.add(1, &[{}]);
            }}
    "#, self.meter_provider, self.name, self.description, self.labels)).unwrap()
    }

    pub fn process(&mut self) -> TokenStream {
        self.check_metric_name();
        let mut code_block = self.build_code_block();

        let mut item_fn = self.item_fn.take().unwrap();

        code_block.stmts.extend_from_slice(&*item_fn.block.stmts);
        item_fn.block.stmts = code_block.stmts;
        TokenStream::from(item_fn.into_token_stream())
    }
}
