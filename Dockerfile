FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo install --debug --path .

FROM alpine:3.15.0 as run

COPY --from=build /usr/local/cargo/bin/server .

CMD ["pwd && ls -a"]

ENTRYPOINT server
