FROM rust:1.56-alpine as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo install --debug --path .

FROM ubuntu:latest as run

WORKDIR /usr/src

COPY --from=build /usr/local/cargo/bin/server /usr/local/bin

ENTRYPOINT ["/usr/local/bin/server"]
