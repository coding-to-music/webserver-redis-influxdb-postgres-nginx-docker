FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY ./src .

RUN cd collector && cargo install --path .

FROM ubuntu:latest as run

WORKDIR /usr/src

# RUN apt-get update
# RUN apt-get -y install libssl-dev

COPY --from=build /usr/local/cargo/bin/collector /usr/local/bin

ENTRYPOINT ["/usr/local/bin/collector"]
