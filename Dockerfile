FROM rust:1.56

WORKDIR /usr/ijagb/webserver
COPY . .

RUN cargo install --path .

CMD ["webserver"]