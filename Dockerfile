FROM rust:1.65-slim as rust

WORKDIR /app
RUN apt-get update && apt-get install -y \
    curl

RUN curl -L https://github.com/LukeMathWalker/cargo-chef/releases/download/v0.1.35/cargo-chef-x86_64-unknown-linux-gnu.tar.gz --output cargo-chef.tar.gz &&\
    tar -zxvf cargo-chef.tar.gz cargo-chef &&\
    mv cargo-chef /usr/local/cargo/bin/cargo-chef
COPY rust-toolchain.toml .
RUN cargo fetch || true

FROM rust as prepare

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust as build

RUN apt-get update && apt-get install -y \
    libsqlite3-dev libssl-dev pkg-config

# Cache dependencies
WORKDIR /app
COPY --from=prepare /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build
COPY . .
RUN cargo build --release

FROM debian:stable-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    sqlite3 \
    ca-certificates
RUN update-ca-certificates

COPY --from=build \
    /app/target/release/qlytics \
    .
COPY run.sh .

ENV RUST_BACKTRACE=1

ENTRYPOINT [ "./run.sh" ]
