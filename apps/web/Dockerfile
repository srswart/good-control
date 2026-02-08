FROM rust:1.88-bookworm AS builder

WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install dioxus-cli --locked

COPY . .

ARG APP__GRAPHQL_ENDPOINT
ENV APP__GRAPHQL_ENDPOINT=${APP__GRAPHQL_ENDPOINT}

WORKDIR /workspace/apps/web
RUN dx build --release --platform web \
    && cp -R /workspace/target/dx/inventory-web/release/web/public /workspace/apps/web/dist

FROM nginx:1.27-alpine

COPY apps/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY apps/web/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
COPY --from=builder /workspace/apps/web/dist /usr/share/nginx/html

EXPOSE 8080
CMD ["/entrypoint.sh"]
