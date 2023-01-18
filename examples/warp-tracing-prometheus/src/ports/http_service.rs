use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::internal::metrics::PrometheusMetricsHandler;

/* Http Metrics Port */

pub fn metrics_port(
    metrics_handler: PrometheusMetricsHandler,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("metrics")
        .and(warp::get())
        .and(warp::any().map(move || metrics_handler.clone()))
        .and_then(MetricsHttpHandler::handler)
}

/* Http Metrics Handler */

pub struct MetricsHttpHandler {}

impl MetricsHttpHandler {
    pub async fn handler(
        metrics_handler: PrometheusMetricsHandler,
    ) -> Result<impl Reply, Infallible> {
        Ok(metrics_handler.metrics())
    }
}
