## Copied from the rust official image, since they don't have a nightly image
# FROM buildpack-deps:stretch

FROM rust:latest

RUN rustup default stable
RUN cargo install cargo-watch
RUN rustc --version

WORKDIR /usr/src/app

EXPOSE 3001

VOLUME ["/usr/local/cargo"]