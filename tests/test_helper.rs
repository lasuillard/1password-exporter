pub(crate) fn get_random_port() -> u16 {
    let listener =
        std::net::TcpListener::bind("127.0.0.0:0").expect("Failed to bind to random port");

    listener
        .local_addr()
        .expect("Failed to get local address")
        .port()
}
