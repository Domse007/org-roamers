# ----------- Stage 1: Build the Vue frontend ------------
FROM node:20.18-alpine AS frontend

WORKDIR /app/frontend

# Copy package files first for better layer caching
COPY web/package*.json ./
RUN npm ci --prefer-offline --no-audit

# Copy source and build
COPY web/ ./
RUN npm run build

# ----------- Stage 2: Build the Rust backend ------------
FROM rust:1-bookworm AS backend

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy entire workspace for build
COPY ./ ./

# Copy frontend build artifacts from the frontend stage
COPY --from=frontend /app/frontend/dist ./web/dist/

# Build the CLI with static assets feature
RUN cargo build --release --bin org-roamers-cli --features static_assets

# ----------- Final stage: Runtime container ------------
FROM debian:bookworm-slim

# Install runtime dependencies including LaTeX for preview generation
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    texlive-latex-base \
    texlive-latex-extra \
    dvisvgm \
    wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binary from backend stage
COPY --from=backend /app/target/release/org-roamers-cli ./org-roamers-cli

# Copy frontend assets (embedded in binary with static_assets feature, but keeping for reference)
COPY --from=frontend /app/frontend/dist ./web/dist/

# Create directories for data and config
RUN mkdir -p /data /config

# Copy entrypoint script
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Expose the default port
EXPOSE 5000

# Set default environment variables
ENV ORG_ROAMERS_ROOT=/data \
    CONFIG_PATH=/config/conf.json \
    HOST=0.0.0.0 \
    PORT=5000

# Health check to ensure server is running
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT}/ || exit 1

ENTRYPOINT ["/entrypoint.sh"]
CMD ["server"]
