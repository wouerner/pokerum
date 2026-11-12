# syntax=docker/dockerfile:1
# ---------- Produção: imagem única (API Rust + SPA Vue) ----------

# 1) Frontend
FROM node:22-bookworm-slim AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# 2) Backend
FROM rust:1-bookworm AS api
WORKDIR /src
# cache de dependências: compila com fontes vazias antes de copiar o código real
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/core/Cargo.toml core/Cargo.toml
COPY backend/api/Cargo.toml api/Cargo.toml
RUN mkdir -p core/src api/src \
 && echo "" > core/src/lib.rs \
 && echo "fn main() {}" > api/src/main.rs && echo "" > api/src/lib.rs \
 && cargo build --release -p pokerum_api \
 && rm -rf core/src api/src
COPY backend/ ./
RUN touch core/src/lib.rs api/src/lib.rs api/src/main.rs \
 && cargo build --release --locked -p pokerum_api \
 && strip target/release/pokerum_api

# 3) Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home /app pokerum \
 && mkdir -p /app /data && chown pokerum /data
WORKDIR /app
COPY --from=api /src/target/release/pokerum_api /app/pokerum_api
COPY --from=web /web/dist /app/web/dist
ENV PORT=8080 \
    DATABASE_URL="sqlite:///data/pokerum.db?mode=rwc" \
    POKERUM_WEB_DIST=/app/web/dist \
    RUST_LOG=info
USER pokerum
VOLUME ["/data"]
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -fsS http://localhost:8080/api/health || exit 1
CMD ["/app/pokerum_api"]
