FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo build --debug && mv ./target/debug/server /usr/src/webserver

FROM alpine:3.15.0 as run

COPY --from=build /usr/src/webserver/server /usr/local/bin/server

ENTRYPOINT /usr/local/bin/server
