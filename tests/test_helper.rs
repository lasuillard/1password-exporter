#![allow(dead_code)]
/// Test helper utilities for unit and integration tests.

pub(crate) const MOCK_OP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/mock_op.bash");

#[macro_export]
macro_rules! test_dir {
    ($path:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", $path)
    };
}

pub(crate) fn get_random_port() -> u16 {
    let listener =
        std::net::TcpListener::bind("127.0.0.0:0").expect("Failed to bind to random port");

    listener
        .local_addr()
        .expect("Failed to get local address")
        .port()
}
