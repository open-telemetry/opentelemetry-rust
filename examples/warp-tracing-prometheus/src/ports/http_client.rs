use std::{convert::Infallible, time::SystemTime};

use warp::{
    reply::{json, with_status},
    Reply, Filter, Rejection,
    http::StatusCode,
};
use opentelemetry::{
    global,
    trace::{Span, TraceContextExt, Tracer},
    Context, Key,
};
use tracing::{event, Level};

use crate::internal::{
    errors::{CustomError, Error},
    metrics::{HttpMetricsBuilder, HTTP_STATUS_CODE, STATUS_KEY, SUCCESS, ERROR},
};
use crate::ports::middleware::{FromRequest, with_interceptor};

/* Http Hello Ports */

pub fn hello_ports(
    http_metrics: HttpMetricsBuilder,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("hello" / String)
        .and(warp::get())
        .and(with_interceptor())
        .and(warp::any().map(move ||http_metrics.clone()))
        .and_then(HttpHelloHandler::hello)
}

/* Http Hello Handlers */

// Set path matching to export in metrics
const HTTP_PATH_HELLO: &str = "/hello/{param}";

pub struct HttpHelloHandler {}

impl HttpHelloHandler {

    /// HttpHelloHandler - hello()
    pub async fn hello(
        param: String,
        from_request: FromRequest,
        http_metrics: HttpMetricsBuilder,
    ) -> Result<impl Reply, Infallible> {
        // Metrics - Start Timer
        let timer = SystemTime::now();

        // Create Tracer and Parent Context
        let tracer = global::tracer("HttpHelloHandler");
        let parent = tracer.start("HttpHelloHandler.hello");
        let parent_cx = Context::current_with_span(parent);

        // Span
        let mut span = tracer.start_with_context("Processor.task", &parent_cx);
        
        // Task processor...
        let (response, status_code) = match param.parse::<i32>() {
            Ok(param) => {
                let data = format!("Yeah! Hello: {}", param);
                let status_code = StatusCode::OK;
                let res = Ok(with_status(json(&data), status_code));

                // Set custom attribute and add events...
                span.set_attribute(HTTP_STATUS_CODE.string(status_code.as_u16().to_string()));
                span.set_attribute(STATUS_KEY.string(SUCCESS));
                span.add_event(
                    SUCCESS,
                    vec![
                        Key::new("param").string(param.to_string()),
                        Key::new("data").string(data),
                    ],
                );

                (res, status_code.as_u16())
            }
            Err(e) => {
                // TODO: option track error here!
                let data = format!("Try again! You informed: {}", param);
                
                // Create Custom Error
                let err = CustomError::from(Error::BadRequest);

                // Prepare Response
                let response = Ok(with_status(json(&data), StatusCode::from_u16(err.code).unwrap()));

                // Custom Event Tracing Error
                let lines = format!(
                    "input: {}, data: {}, status_code: {}",
                    param, data, err.code.to_string(),
                );

                event!(
                    Level::ERROR,
                    "{:?}",
                    CustomError::tracing_fmt("HttpHelloHandler.hello".to_owned(), lines, e.to_string())
                );

                // Custom Attributes and Add Events to Span
                span.set_attribute(HTTP_STATUS_CODE.string(err.code.to_string()));
                span.set_attribute(STATUS_KEY.string(ERROR));
                span.add_event(
                    ERROR,
                    vec![
                        Key::new("param").string(param.to_string()),
                        Key::new("data").string(data),
                        Key::new("content").string(e.to_string()),
                    ],
                );

                (response, err.code)
            }
        };

        // Metrics - Export
        http_metrics.http_request_counter(
            &parent_cx,
            from_request.method(),
            HTTP_PATH_HELLO,
            status_code.to_string().as_str(),
        );

        http_metrics.http_request_latency(
            &parent_cx,
            from_request.method(),
            HTTP_PATH_HELLO,
            timer
                .elapsed()
                .map(|t| t.as_secs_f64() * 1000.0)
                .unwrap_or_default(),
        );

        response
    }
}
