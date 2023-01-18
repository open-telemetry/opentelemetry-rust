use std::convert::Infallible;
use warp::{Filter, Rejection};

use crate::internal::metrics::PrometheusMetricsHandler;

/* Http Metrics Port */

pub fn metrics_port(
    metrics_handler: PrometheusMetricsHandler,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::path!("metrics")
        .and(warp::get())
        .and(warp::any().map(move || metrics_handler.clone()))
        .and_then(MetricsHttpHandler::handler)
}

/* Http Metrics Handler */

pub struct MetricsHttpHandler {}

impl MetricsHttpHandler {
    async fn handler(metrics_handler: PrometheusMetricsHandler) -> Result<String, Infallible> {
        Ok(metrics_handler.metrics())
    }
}
