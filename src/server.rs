use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{header::CONTENT_TYPE, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use prometheus::{Encoder, TextEncoder};
use tokio::net::TcpListener;

use crate::{command_executor::OpCommandExecutor,
            metrics_collector::{Metrics, OpMetricsCollector}};

async fn serve(
    _req: Request<impl hyper::body::Body>,
    metrics: Vec<Metrics>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Collect all metrics
    let command_executor = OpCommandExecutor {};
    let metrics_collector = OpMetricsCollector::new(Box::new(command_executor));
    // TODO: Only collect required metrics (read from config)
    metrics_collector.collect(metrics);

    // Encode to Prometheus format
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    // Respond with the metrics
    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Full::new(Bytes::from(buffer)))
        .unwrap();

    Ok(response)
}

pub(crate) async fn run_server(
    host: String,
    port: u16,
    metrics: Vec<Metrics>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on http://{}", addr);
    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);
        let inner = metrics.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service_fn(move |req| serve(req, inner.clone())))
                .await
            {
                log::error!("Error serving connection: {:?}", err);
            }
        });
    }
}
