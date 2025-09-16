use axum::{
    http::{Request, Response, HeaderValue},
    body::Body,
};
use tower::{Layer, Service};
use uuid::Uuid;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Clone)]
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestId<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for RequestIdService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let request_id = req.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        req.extensions_mut().insert(request_id.clone());

        let request_id_for_response = request_id.clone();

        let future = self.inner.call(req);

        Box::pin(async move {
            let mut response = future.await?;

            if let Ok(header_value) = HeaderValue::from_str(&request_id_for_response) {
                response.headers_mut().insert("x-request-id", header_value);
            }

            Ok(response)
        })
    }
}