FROM rust:1.56-alpine as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo install --debug --path .

FROM alpine:3.15.0 as run

WORKDIR /usr/src

RUN apk update && apk add \
    git \
    make \
    gcc \
    g++ \
    zlib \
    zlib-dev \
    python3 \
    ldc 

COPY --from=build /usr/local/cargo/bin/server /usr/local/bin

ENTRYPOINT ["/usr/local/bin/server"]
