use futures::FutureExt;
use futures::future::BoxFuture;
use http::{Request, Response};
use http_body::Body;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use tracing::info;

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Body + Send + 'static,
    S::Error: std::fmt::Debug + Send + Sync + 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let path = req.uri().path().to_string();
        let start = Instant::now();

        let fut = self.inner.call(req);

        async move {
            let res = fut.await;
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

            let time_str = if elapsed_ms < 100.0 {
                format!("\x1b[32m{:.3} ms\x1b[0m", elapsed_ms)
            } else if elapsed_ms < 300.0 {
                format!("\x1b[33m{:.3} ms\x1b[0m", elapsed_ms)
            } else {
                format!("\x1b[31m{:.3} ms\x1b[0m", elapsed_ms)
            };

            info!(
                target: "middleware::execution_time",
                path = %path,
                time = %time_str,
                "🕒 Execution time: {} [{}]", time_str, path
            );

            res
        }
        .boxed()
    }
}
