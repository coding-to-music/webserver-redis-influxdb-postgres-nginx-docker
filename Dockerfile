FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo build --release

FROM alpine:3.15.0 as run

COPY --from=build /usr/src/webserver/target/release/server /usr/local/bin/server

ENTRYPOINT /usr/local/bin/server
