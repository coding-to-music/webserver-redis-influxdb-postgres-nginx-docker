FROM rust:1.56

WORKDIR /usr/src

COPY . .

RUN cargo install --path ./server

CMD ["./server"]
