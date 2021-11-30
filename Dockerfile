FROM rust:1.56 as rust-builder
WORKDIR /usr/src/webserver
COPY ./server/Cargo.toml .
COPY ./server/Cargo.lock .
RUN mkdir ./server/src && echo 'fn main() { println!("Dummy!"); }' > ./server/src/main.rs
RUN cargo build --release 
RUN rm -rf ./src
COPY ./src ./src
RUN touch -a -m ./src/main.rs
RUN cargo build --release

FROM alpine
COPY --from=rust-builder /usr/src/webserver/target/release/server /usr/local/bin/
WORKDIR /usr/local/bin
CMD ["app"]
