FROM rust:1.85-bookworm AS builder

WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libsqlite3-dev \
        pkg-config \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build -p inventory-server --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libsqlite3-0 \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /workspace/target/release/inventory-server /app/inventory-server

ENV APP__SERVER__HOST=0.0.0.0 \
    APP__SERVER__PORT=3000

EXPOSE 3000

CMD ["/app/inventory-server"]
