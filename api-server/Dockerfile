## Copied from the rust official image, since they don't have a nightly image
# FROM buildpack-deps:stretch

FROM rust:latest

RUN rustup toolchain nightly && rustup default nightly && rustup update
RUN cargo install cargo-watch

CMD cargo watch -c -x run

WORKDIR /usr/src/app

EXPOSE 3001

VOLUME ["/usr/local/cargo"]