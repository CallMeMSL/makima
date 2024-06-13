FROM rust:latest

WORKDIR /usr/src/makima
COPY . .
RUN cargo build --release
CMD cargo run --release
