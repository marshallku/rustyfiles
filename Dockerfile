FROM rust:1.95-alpine AS base

WORKDIR /usr/src/rustyfiles

RUN set -eux; \
    apk add --no-cache musl-dev pkgconfig libressl-dev; \
    rm -rf $CARGO_HOME/registry

COPY Cargo.* .

RUN mkdir src && \
    echo 'fn main() {println!("Hello, world!");}' > src/main.rs && \
    cargo build --release && \
    rm target/release/rustyfiles* && \
    rm target/release/deps/rustyfiles* && \
    rm -rf src

FROM base AS builder

COPY src src
RUN cargo build --release

FROM alpine:3.14

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/rustyfiles/target/release/rustyfiles .

EXPOSE 41890

CMD ["./rustyfiles"]