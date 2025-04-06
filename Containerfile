FROM rust:slim-bookworm as builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    file=* libssl-dev=* make=* pkg-config=* && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src
RUN cargo new --bin dray
WORKDIR /usr/src/dray
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl=* && rm -rf /var/lib/apt/lists/*
RUN groupadd -r dray && useradd -r -s /bin/false -g dray dray
COPY --from=builder /usr/local/cargo/bin/dray /usr/local/bin/dray
USER dray
CMD ["dray"]
