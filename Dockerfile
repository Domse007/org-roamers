# ----------- Stage 1: Build the Vue frontend ------------
FROM node:20 AS frontend

WORKDIR /app/frontend

COPY web/ ./
RUN npm install
RUN npm run build

# ----------- Stage 2: Build the Rust backend ------------
FROM rust:latest AS backend

WORKDIR /app

COPY ./ ./backend

RUN cd backend && cargo build --release --bin server

# ----------- Final stage: Runtime container ------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend  /app/backend/target/release/server ./server
COPY --from=backend  /app/backend/server_conf.json      ./server_conf.json
COPY --from=frontend /app/frontend/dist                 ./web/dist/

# Expose a port (change as needed)
EXPOSE 5000

# Define a runtime argument for data directory (via env var or CMD)
ENV DATA_DIR=/data

CMD [ "./server", "/data" ]
