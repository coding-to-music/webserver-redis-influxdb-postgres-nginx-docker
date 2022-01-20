FROM rust:1.58 AS chef
RUN cargo install cargo-chef
RUN pwd

FROM chef AS planner
COPY . .
RUN pwd
RUN ls
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies - this is the caching Docker layer!
FROM chef AS builder 
COPY --from=planner /server/recipe.json recipe.json

RUN pwd
RUN ls
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin server

FROM ubuntu:latest as run
COPY --from=builder /app/target/release/server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server"]