use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;

pub struct Metrics {
    pub router: Router,                         // provides GET /metrics
    pub layer: PrometheusMetricLayer<'static>,  // prometheus middleware
}

pub fn init() -> Metrics {
    let (layer, handle) = PrometheusMetricLayer::pair(); // handle type is private; keep it internal
    let router = Router::new().route("/metrics", {
        let h = handle.clone();
        get(move || async move { h.render() })
    });
    Metrics { router, layer }
}
