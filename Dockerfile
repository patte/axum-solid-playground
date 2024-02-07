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
# install ca-certificates fuse3 sqlite3 for litefs
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends libssl-dev && \
    apt-get install -y ca-certificates fuse3 sqlite3 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# litefs
COPY --from=flyio/litefs:0.5 /usr/local/bin/litefs /usr/local/bin/litefs
COPY litefs.yml /etc/litefs.yml

COPY --from=backend-builder /app/server/target/release/axum-solid-playground /app/main 

# see exec.cmd in litefs.yml
ENTRYPOINT litefs mount
