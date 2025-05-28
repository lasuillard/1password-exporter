use std::{process::Command, thread};

use assert_cmd::cargo::cargo_bin;
use test_helper::MOCK_OP;

mod test_helper;

#[tokio::test]
async fn test_metrics_serving() {
    let port = test_helper::get_random_port();
    thread::scope(|s| {
        s.spawn(|| {
            Command::new(cargo_bin!(env!("CARGO_PKG_NAME")))
                .args(&[
                    "--log-level",
                    "DEBUG",
                    "--port",
                    &port.to_string(),
                    "--op-path",
                    MOCK_OP,
                    "--metrics",
                    "account,build-info,document,group,item,service-account,user,vault",
                    "--service-account-token",
                    "ops_blahblah",
                ])
                .spawn()
                .expect("Failed to start the exporter process");
        });
    });

    // Wait for the server to start
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let body = reqwest::get(format!("http://localhost:{port}/metrics"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(body, include_str!(test_dir!("expected_metrics.txt")),);
}
