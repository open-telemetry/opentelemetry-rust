#[allow(unused, missing_docs)]
pub mod jaeger_api_v2;

#[allow(missing_docs)]
pub mod jaeger_client {
    use crate::testing::jaeger_api_v2::query_service_client::QueryServiceClient;
    use crate::testing::jaeger_api_v2::{
        FindTracesRequest, GetServicesRequest, GetTraceRequest, Span as JaegerSpan,
        TraceQueryParameters,
    };
    use tonic::transport::Channel;

    #[derive(Debug)]
    pub struct JaegerTestClient {
        query_service_client: QueryServiceClient<Channel>,
    }

    impl JaegerTestClient {
        pub fn new(jaeger_url: &'static str) -> JaegerTestClient {
            let channel = Channel::from_static(jaeger_url).connect_lazy();

            JaegerTestClient {
                query_service_client: QueryServiceClient::new(channel),
            }
        }

        /// Check if the jaeger contains the service
        pub async fn contain_service(&mut self, service_name: &String) -> bool {
            self.query_service_client
                .get_services(GetServicesRequest {})
                .await
                .unwrap()
                .get_ref()
                .services
                .iter()
                .any(|svc_name| *svc_name == *service_name)
        }

        /// Find trace by trace id.
        /// Note that `trace_id` should be a u128 in hex.
        pub async fn get_trace(&mut self, trace_id: String) -> Vec<JaegerSpan> {
            let trace_id = u128::from_str_radix(trace_id.as_ref(), 16).expect("invalid trace id");
            let mut resp = self
                .query_service_client
                .get_trace(GetTraceRequest {
                    trace_id: trace_id.to_be_bytes().into(),
                })
                .await
                .unwrap();

            return if let Some(spans) = resp
                .get_mut()
                .message()
                .await
                .expect("jaeger returns error")
            {
                spans.spans
            } else {
                vec![]
            };
        }

        /// Find traces belongs the service.
        /// It assumes the service exists.
        pub async fn find_traces_from_services(&mut self, service_name: &str) -> Vec<JaegerSpan> {
            let request = FindTracesRequest {
                query: Some(TraceQueryParameters {
                    service_name: service_name.to_owned(),
                    ..Default::default()
                }),
            };
            self.query_service_client
                .find_traces(request)
                .await
                .unwrap()
                .get_mut()
                .message()
                .await
                .expect("jaeger returns error")
                .unwrap_or_default()
                .spans
        }
    }
}
