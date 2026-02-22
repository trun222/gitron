# Stage 1: Build frontend
FROM node:22-slim AS frontend
WORKDIR /app
ARG VITE_BASE=""
ENV VITE_BASE=${VITE_BASE}
COPY package.json package-lock.json ./
RUN npm ci
COPY src/ src/
COPY static/ static/
COPY svelte.config.js tsconfig.json vite.config.js ./
RUN npm run build

# Stage 2: Build Rust server
FROM rust:1-slim AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml
# Create a dummy src-tauri/src to satisfy workspace resolution
RUN mkdir -p src-tauri/src && echo "fn main() {}" > src-tauri/src/main.rs && echo "" > src-tauri/src/lib.rs
RUN cargo build -p gitron-server --release
RUN rm -rf src-tauri/src

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gitron-server /usr/local/bin/gitron-server
COPY --from=frontend /app/build /app/frontend

EXPOSE 9417
ENTRYPOINT ["gitron-server", "--host", "0.0.0.0", "--frontend-dir", "/app/frontend"]
