# Desenvolvimento da API: código montado como volume, recompila ao salvar
FROM rust:1-bookworm
RUN rustup component add clippy rustfmt && cargo install cargo-watch --locked
WORKDIR /app/backend
EXPOSE 8080
CMD ["cargo", "watch", "-w", "core", "-w", "api", "-x", "run -p pokerum_api"]
