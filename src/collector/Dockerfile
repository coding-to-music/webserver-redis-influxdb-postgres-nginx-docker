FROM lukemathwalker/cargo-chef:latest-rust-1.58.0 AS chef
WORKDIR /webserver

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies - this is the caching Docker layer!
FROM chef AS builder 
COPY --from=planner /webserver/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin collector

FROM ubuntu:latest as run

RUN apt-get update
RUN apt-get -y install ca-certificates openssl

COPY --from=builder /webserver/target/release/collector /usr/local/bin
ENTRYPOINT ["/usr/local/bin/collector", "weather"]
