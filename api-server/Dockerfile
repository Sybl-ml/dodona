## Copied from the rust official image, since they don't have a nightly image
# FROM buildpack-deps:stretch

FROM rust:latest

RUN rustup default nightly
RUN cargo install cargo-watch
RUN rustc --version

# CMD cargo watch -c -x run

WORKDIR /usr/src/app

EXPOSE 3001

VOLUME ["/usr/local/cargo"]