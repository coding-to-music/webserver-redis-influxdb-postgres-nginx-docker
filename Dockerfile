FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo install --debug --path .

FROM alpine:3.15.0 as run

COPY --from=build /usr/local/bin/server /usr/local/bin/server

ENTRYPOINT ["./usr/local/bin/server"]
