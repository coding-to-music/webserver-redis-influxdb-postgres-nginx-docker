FROM rust:1.58 AS chef
WORKDIR webserver
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN pwd
RUN ls
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies - this is the caching Docker layer!
FROM chef AS builder 
COPY --from=planner /webserver/recipe.json recipe.json
RUN ls
RUN pwd
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin server

FROM ubuntu:latest as run

RUN apt-get update
RUN apt-get -y install ca-certificates openssl

COPY --from=builder /webserver/target/release/server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server"]
