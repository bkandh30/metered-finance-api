use http::header::HeaderName;
pub use tower_http::request_id::{
    MakeRequestUuid,
    PropagateRequestIdLayer,
    SetRequestIdLayer
};

pub fn request_id_layers() -> (
    PropagateRequestIdLayer,
    SetRequestIdLayer<MakeRequestUuid>,
) {
    let header = HeaderName::from_static("x-request-id");
    (
        PropagateRequestIdLayer::new(header.clone()),
        SetRequestIdLayer::new(header, MakeRequestUuid),
    )
}