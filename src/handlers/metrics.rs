use axum::{
    response::IntoResponse,
    http::StatusCode,
};
use once_cell::sync::Lazy;
use axum_prometheus::{
    PrometheusMetricLayer,
    metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle},
};

static PROMETHEUS_HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| {
    init_metrics()
});

pub fn init_metrics() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}


pub fn create_metrics_layer() -> PrometheusMetricLayer<'static> {
    PrometheusMetricLayer::new()
}

pub async fn metrics_handler(handle: PrometheusHandle) -> impl IntoResponse {
    (StatusCode::OK, handle.render())
}