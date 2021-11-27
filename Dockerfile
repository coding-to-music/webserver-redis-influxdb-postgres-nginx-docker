# Build stage

FROM rust:1.56 as build

WORKDIR /usr/src/webserver
COPY . .

RUN cd server && cargo install --debug --verbose --path .

ENTRYPOINT server
