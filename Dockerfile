FROM rust:1.56 as build

WORKDIR /usr/src/webserver

COPY . .

RUN cd server && cargo install --debug --path .

FROM alpine:3.15.0 as run

WORKDIR /usr/src

COPY --from=build /usr/local/cargo/bin/server /usr/local/bin

RUN echo $(pwd)
RUN echo $(ls -a)
RUN echo $(stat server)

ENTRYPOINT ["./server"]
