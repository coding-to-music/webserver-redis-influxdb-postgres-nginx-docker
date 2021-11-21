# Build stage

FROM rust:1.56 as build

WORKDIR /usr/src/server

COPY . .

RUN cargo build --release

RUN cargo install --path .

# Execution stage

FROM alpine:latest

COPY --from=build /usr/local/cargo/bin/server /usr/local/bin/server

CMD ["server"]