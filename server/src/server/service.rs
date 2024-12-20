//! Hyper service implementation

use std::{fs, future::Future, pin::Pin};

use http_body_util::Full;
use hyper::{
    body::{self, Bytes},
    service::Service,
    Method, Request, Response, StatusCode,
};

#[derive(Default)]
pub struct SpjortService;

impl Service<Request<body::Incoming>> for SpjortService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<body::Incoming>) -> Self::Future {
        let response = Response::builder();

        let res = match *req.method() {
            Method::GET => match req.uri().path() {
                "/" => response
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::from_static(b"Welcome to Spjorts!"))),
                _ => response
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from_static(b"Not Found"))),
            },
            _ => response
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Full::new(Bytes::from_static(b"Method Not Allowed"))),
        };

        Box::pin(async { res })
    }
}
