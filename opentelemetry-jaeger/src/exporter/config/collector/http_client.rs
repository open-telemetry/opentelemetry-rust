use crate::Error::NoHttpClient;
#[cfg(feature = "surf_collector_client")]
use async_trait::async_trait;
#[cfg(any(
feature = "reqwest_blocking_collector_client",
feature = "reqwest_collector_client"
))]
use headers::authorization::Credentials;
#[cfg(feature = "isahc_collector_client")]
use isahc::config::Configurable;
use opentelemetry_http::HttpClient as OtelHttpClient;
#[cfg(feature = "surf_collector_client")]
use std::convert::TryInto;
use std::time::Duration;

#[derive(Debug)]
#[cfg(feature = "surf_collector_client")]
struct BasicAuthMiddleware(surf::http::auth::BasicAuth);

#[async_trait]
#[cfg(feature = "surf_collector_client")]
impl surf::middleware::Middleware for BasicAuthMiddleware {
    async fn handle(
        &self,
        mut req: surf::Request,
        client: surf::Client,
        next: surf::middleware::Next<'_>,
    ) -> surf::Result<surf::Response> {
        req.insert_header(self.0.name(), self.0.value());
        next.run(req, client).await
    }
}

#[derive(Debug)]
pub(crate) enum CollectorHttpClient {
    None,
    Custom(Box<dyn OtelHttpClient>),
    #[cfg(feature = "isahc_collector_client")]
    Isahc,
    #[cfg(feature = "surf_collector_client")]
    Surf,
    #[cfg(feature = "reqwest_collector_client")]
    Reqwest,
    #[cfg(feature = "reqwest_blocking_collector_client")]
    ReqwestBlocking,
}

impl CollectorHttpClient {
    // try to build a build in http client if users chose one. If none available return NoHttpClient error
    #[allow(unused_variables)] // if the user enabled no build in client features. all parameters are unsed.
    pub(crate) fn build_client(
        self,
        collector_username: Option<String>,
        collector_password: Option<String>,
        collector_timeout: Duration,
    ) -> Result<Box<dyn OtelHttpClient>, crate::Error> {
        match self {
            CollectorHttpClient::Custom(client) => Ok(client),
            CollectorHttpClient::None => Err(NoHttpClient),
            #[cfg(feature = "isahc_collector_client")]
            CollectorHttpClient::Isahc => {
                let mut builder = isahc::HttpClient::builder().timeout(collector_timeout);

                if let (Some(username), Some(password)) = (collector_username, collector_password) {
                    builder = builder
                        .authentication(isahc::auth::Authentication::basic())
                        .credentials(isahc::auth::Credentials::new(username, password));
                }

                let client = builder.build().map_err(|err| {
                    crate::Error::ConfigError {
                        config_name: "http_client",
                        pipeline_name: "collector",
                        reason: format!("cannot create isahc http client, {}", err),
                    }
                })?;
                Ok(Box::new(client))
            }
            #[cfg(feature = "surf_collector_client")]
            CollectorHttpClient::Surf => {
                let client: surf::Client = surf::Config::new()
                    .set_timeout(Some(collector_timeout))
                    .try_into()
                    .map_err(|err| crate::Error::ConfigError {
                        pipeline_name: "collector",
                        config_name: "http_client",
                        reason: format!("cannot create surf client. {}", err),
                    })?;

                let client = if let (Some(username), Some(password)) =
                (collector_username, collector_password)
                {
                    let auth = surf::http::auth::BasicAuth::new(username, password);
                    client.with(BasicAuthMiddleware(auth))
                } else {
                    client
                };

                Ok(Box::new(client))
            }
            #[cfg(feature = "reqwest_blocking_collector_client")]
            CollectorHttpClient::ReqwestBlocking => {
                let mut builder =
                    reqwest::blocking::ClientBuilder::new().timeout(collector_timeout);
                if let (Some(username), Some(password)) = (collector_username, collector_password) {
                    let mut map = http::HeaderMap::with_capacity(1);
                    let auth_header_val =
                        headers::Authorization::basic(username.as_str(), password.as_str());
                    map.insert(http::header::AUTHORIZATION, auth_header_val.0.encode());
                    builder = builder.default_headers(map);
                }
                let client = builder.build().map_err::<crate::Error, _>(|err| {
                    crate::Error::ConfigError {
                        pipeline_name: "http_client",
                        config_name: "collector",
                        reason: format!("cannot create reqwest blocking http client, {}", err),
                    }
                })?;
                Ok(Box::new(client))
            }
            #[cfg(feature = "reqwest_collector_client")]
            CollectorHttpClient::Reqwest => {
                let mut builder = reqwest::ClientBuilder::new().timeout(collector_timeout);
                if let (Some(username), Some(password)) = (collector_username, collector_password) {
                    let mut map = http::HeaderMap::with_capacity(1);
                    let auth_header_val =
                        headers::Authorization::basic(username.as_str(), password.as_str());
                    map.insert(http::header::AUTHORIZATION, auth_header_val.0.encode());
                    builder = builder.default_headers(map);
                }
                let client = builder.build().map_err::<crate::Error, _>(|err| {
                    crate::Error::ConfigError {
                        pipeline_name: "http_client",
                        config_name: "collector",
                        reason: format!("cannot create reqwest http client, {}", err),
                    }
                })?;
                Ok(Box::new(client))
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod test_http_client {
    use async_trait::async_trait;
    use bytes::Bytes;
    use http::{Request, Response};
    use opentelemetry_http::{HttpClient, HttpError};
    use std::fmt::Debug;

    pub(crate) struct TestHttpClient;

    impl Debug for TestHttpClient {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("test http client")
        }
    }

    #[async_trait]
    impl HttpClient for TestHttpClient {
        async fn send(&self, _request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
            Err("wrong uri set in http client".into())
        }
    }
}

#[cfg(test)]
#[cfg(all(feature = "collector_client", feature = "rt-tokio"))]
mod collector_client_tests {
    use crate::config::build_config_and_process;
    use crate::config::collector::http_client::test_http_client;
    use crate::exporter::thrift::jaeger::Batch;
    use crate::new_collector_pipeline;
    use opentelemetry::runtime::Tokio;
    use opentelemetry::sdk::Resource;
    use opentelemetry::trace::TraceError;
    use opentelemetry::KeyValue;

    #[test]
    fn test_bring_your_own_client() -> Result<(), TraceError> {
        let invalid_uri_builder = new_collector_pipeline()
            .with_endpoint("localhost:6831")
            .with_http_client(test_http_client::TestHttpClient);
        let sdk_provided_resource =
            Resource::new(vec![KeyValue::new("service.name", "unknown_service")]);
        let (_, process) = build_config_and_process(sdk_provided_resource, None, None);
        let mut uploader = invalid_uri_builder.build_uploader::<Tokio>()?;
        let res = futures_executor::block_on(async {
            uploader
                .upload(Batch::new(process.into(), Vec::new()))
                .await
        });
        assert_eq!(
            format!("{:?}", res.err().unwrap()),
            "Other(\"wrong uri set in http client\")"
        );

        let valid_uri_builder = new_collector_pipeline()
            .with_http_client(test_http_client::TestHttpClient)
            .build_uploader::<Tokio>();

        assert!(valid_uri_builder.is_ok());
        Ok(())
    }
}
