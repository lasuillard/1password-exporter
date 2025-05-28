use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{header::CONTENT_TYPE, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use prometheus::{Encoder, TextEncoder};
use tokio::net::TcpListener;

use crate::{command_executor::OpCommandExecutor,
            metrics_collector::{Metrics, OpMetricsCollector}};

async fn serve_metrics(
    metrics: Vec<Metrics>,
    op_path: String,
    service_account_token: Option<String>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Collect all metrics
    let command_executor = OpCommandExecutor::new(op_path, service_account_token);
    let metrics_collector = OpMetricsCollector::new(Box::new(command_executor));
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

async fn serve(
    req: Request<impl hyper::body::Body>,
    metrics: Vec<Metrics>,
    op_path: String,
    service_account_token: Option<String>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/metrics") => {
            serve_metrics(metrics, op_path, service_account_token).await
        }
        _ => {
            let response = Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap();
            Ok(response)
        }
    }
}

pub(crate) async fn run_server(
    host: String,
    port: u16,
    metrics: Vec<Metrics>,
    op_path: String,
    service_account_token: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    log::info!("Enabled metrics: {:?}", metrics);
    log::info!("Using 1Password CLI: {}", op_path);
    if service_account_token.is_some() {
        log::warn!("Service account token explicitly set.");
    }
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening on {}", listener.local_addr()?);
    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let metrics = metrics.clone();
        let op_path = op_path.clone();
        let service_account_token = service_account_token.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        serve(
                            req,
                            metrics.clone(),
                            op_path.clone(),
                            service_account_token.clone(),
                        )
                    }),
                )
                .await
            {
                log::error!("Error serving connection: {:?}", err);
            }
        });
    }
}
