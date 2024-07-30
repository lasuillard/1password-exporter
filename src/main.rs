use bytes::Bytes;
use http_body_util::Full;
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use lazy_static::lazy_static;
use prometheus::register_int_gauge_vec;
use prometheus::IntGaugeVec;
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod op;

lazy_static! {
    static ref OP_RATELIMIT_USED: IntGaugeVec = register_int_gauge_vec!(
        "op_ratelimit_used",
        "1Password API rate limit used.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_RATELIMIT_LIMIT: IntGaugeVec = register_int_gauge_vec!(
        "op_ratelimit_limit",
        "1Password API rate limit.",
        &["type", "action"]
    )
    .unwrap();
    static ref OP_RATELIMIT_REMAINING: IntGaugeVec = register_int_gauge_vec!(
        "op_ratelimit_remaining",
        "1Password API rate limit remaining.",
        &["type", "action"]
    )
    .unwrap();
}

fn collect_metrics() {
    let rate_limit = op::read_ratelimit();
    for rl in rate_limit {
        OP_RATELIMIT_LIMIT
            .with_label_values(&[&rl.type_, &rl.action])
            .set(rl.limit as i64);
        OP_RATELIMIT_USED
            .with_label_values(&[&rl.type_, &rl.action])
            .set(rl.used as i64);
        OP_RATELIMIT_REMAINING
            .with_label_values(&[&rl.type_, &rl.action])
            .set(rl.remaining as i64);
    }
}

async fn serve(
    _req: Request<impl hyper::body::Body>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    collect_metrics();
    let metric_families = prometheus::gather();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Full::new(Bytes::from(buffer)))
        .unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = "0.0.0.0:9999".parse()?;

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service_fn(serve))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
