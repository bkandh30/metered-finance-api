use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;

pub struct Metrics {
    pub router: Router,
    pub layer: PrometheusMetricLayer<'static>,
}

pub fn init() -> Metrics {
    let (layer, handle) = PrometheusMetricLayer::pair();
    let router = Router::new().route("/metrics", {
        let h = handle.clone();
        get(move || async move { h.render() })
    });
    Metrics { router, layer }
}
