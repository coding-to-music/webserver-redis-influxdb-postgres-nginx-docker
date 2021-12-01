FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo install --path .

FROM ubuntu:latest as run

WORKDIR /usr/src

COPY --from=build /usr/local/cargo/bin/server /usr/local/bin

ENTRYPOINT ["/usr/local/bin/server"]
