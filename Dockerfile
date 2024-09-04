FROM rustlang/rust:nightly-bookworm-slim AS builder

WORKDIR /build
COPY . .
RUN cargo build --release

FROM 1password/op:2 AS runtime

COPY --from=builder /build/target/release/onepassword-exporter /usr/local/bin/onepassword-exporter

ENTRYPOINT ["onepassword-exporter"]
CMD ["--host", "0.0.0.0"]
