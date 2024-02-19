# Stage 1: Build frontend
FROM node:21.6.1-slim AS frontend-builder

WORKDIR /app/client

COPY client/package.json ./
COPY client/package-lock.json ./
#RUN npm install
RUN npm ci --prefer-offline

COPY client .
RUN npm run build


# Stage 2: Build backend
FROM rust:1.75.0-bookworm AS backend-builder

RUN cargo new --bin app/server
WORKDIR /app/server
COPY server/Cargo.toml ./
COPY server/Cargo.lock ./
RUN cargo build --release  

COPY server/src ./src
COPY server/migrations ./migrations
COPY --from=frontend-builder /app/client/dist /app/client/dist
RUN touch src/main.rs
RUN cargo build --release

# Stage 3: Final image
FROM debian:bookworm-slim

# install libssl
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/server/target/release/axum-solid-playground /app/main 

EXPOSE 3000
CMD "/app/main"
