# Rust Backend & Cloud Deployment Architecture

This document describes the high-performance Rust backend architecture using Axum and the Docker deployment guidelines required to run seamlessly on cloud platforms like Render and Railway.

---

## 1. Rust Axum Architecture Overview

### Why Rust + Axum?
1. **Speed & Latency**: Sub-millisecond response times for scraped HTML parsing and JSON serialization.
2. **Minimal Memory Footprint**: Uses ~15MB RAM idle, ~35MB under load (fits easily within Render's 512MB free tier).
3. **Native Concurrency**: Tokio asynchronous runtime handles hundreds of concurrent image proxy streams without thread exhaustion.

### Essential Dependencies (`Cargo.toml`):
```toml
[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json", "stream"] }
scraper = "0.20"
utoipa = { version = "4.2", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "7.1", features = ["axum"] }
```

---

## 2. Server Routing & Static File Serving

Serve both the REST API endpoints and the single-page application frontend from one unified binary:

```rust
use axum::{Router, routing::get};
use tower_http::{cors::CorsLayer, services::ServeFile};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        // API Endpoints
        .route("/api/latest", get(handlers::get_latest))
        .route("/api/search", get(handlers::search_manga))
        .route("/api/manga/:slug", get(handlers::get_manga_detail))
        .route("/api/chapter", get(handlers::get_chapter))
        .route("/api/proxy-image", get(handlers::proxy_image))
        // Serve Frontend SPA
        .route_service("/", ServeFile::new("static/index.html"))
        .fallback_service(ServeFile::new("static/index.html"))
        .layer(CorsLayer::permissive());

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## 3. Resolving the `weak version GLIBC_2.39 not found` Deployment Error

### The Root Cause:
If the `Dockerfile` uses `FROM rust:latest` (built on Debian Trixie/Testing with `glibc 2.39`), but the runtime container uses `FROM debian:bookworm-slim` (which only has `glibc 2.36`), the compiled binary will fail to execute with:
```
manga_api: /lib/x86_64-linux-gnu/libc.so.6: weak version 'GLIBC_2.39' not found (required by manga_api)
```

### The Fix: Pin Matching Base Images
Ensure both the builder stage and the runner stage use the exact same Debian release (`bookworm`):

```dockerfile
# Stage 1: Build the optimized binary
FROM rust:1.83-bookworm AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source code and static assets
COPY src ./src
COPY static ./static
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime Image (Minimal size, matching libc)
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies (curl and certificates for scraper fallback)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/manga_api /app/manga_api
COPY static /app/static

ENV PORT=8000
EXPOSE 8000

CMD ["./manga_api"]
```

---

## 4. Deploying to Render & Railway

### Render Deployment Configuration:
1. **Repository**: Connect GitHub repo.
2. **Environment**: Docker.
3. **Port**: Automatically binds to `$PORT` (typically 10000 on Render).
4. **Auto-Deploy**: Enabled on `main` branch.

### Railway Deployment Configuration (`railway.json`):
```json
{
  "$schema": "https://railway.app/railway.schema.json",
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile"
  },
  "deploy": {
    "startCommand": "./manga_api",
    "restartPolicyType": "ON_FAILURE",
    "restartPolicyMaxRetries": 5
  }
}
```

