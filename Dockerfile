FROM rust:1.88-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/user_management_backend_rust /app/user_management_backend_rust
COPY migrations /app/migrations

ENV APP_HOST=0.0.0.0
ENV APP_PORT=8080

EXPOSE 8080
CMD ["/app/user_management_backend_rust"]
