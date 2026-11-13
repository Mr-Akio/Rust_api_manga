# Stage 1: Build binary
FROM rust:1-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first to cache layer
COPY Cargo.toml Cargo.lock ./

# Create dummy main to pre-build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source code
COPY src ./src

# Touch to trigger recompile of actual code
RUN touch src/main.rs && cargo build --release

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install curl (for Cloudflare bypass) and ca-certificates (for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary and static assets
COPY --from=builder /app/target/release/manga_api /usr/local/bin/manga_api
COPY static ./static

ENV PORT=8000
EXPOSE 8000

CMD ["manga_api"]
