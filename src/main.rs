mod models;
mod scraper;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use models::{
    ChapterQuery, ListQuery, MangaQuery, ProxyQuery, SearchQuery,
};
use scraper::MangaScraper;

const CACHE_TTL_SECS: u64 = 300; // 5 minutes

#[derive(Clone)]
pub struct AppState {
    pub scraper: MangaScraper,
    pub cache: Arc<RwLock<HashMap<String, (Value, Instant)>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scraper: MangaScraper::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_cached(&self, key: &str) -> Option<Value> {
        let read = self.cache.read().await;
        if let Some((val, instant)) = read.get(key) {
            if instant.elapsed() < Duration::from_secs(CACHE_TTL_SECS) {
                return Some(val.clone());
            }
        }
        None
    }

    pub async fn set_cached(&self, key: String, val: Value) {
        let mut write = self.cache.write().await;
        if write.len() > 500 {
            write.clear();
        }
        write.insert(key, (val, Instant::now()));
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/reader", get(serve_index))
        .route("/docs", get(serve_index))
        .nest_service("/static", ServeDir::new("static"))
        .route("/api/manga/{slug}", get(get_manga_by_slug))
        .route("/api/manga", get(get_manga_by_query))
        .route("/api/search", get(search_manga))
        .route("/api/latest", get(list_latest))
        .route("/api/chapter", get(get_chapter_pages))
        .route("/api/proxy-image", get(proxy_image))
        .route("/api/debug", get(debug_scrape))
        .layer(cors)
        .with_state(state);

    let port_str = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let port: u16 = port_str.parse().unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🦀 Rust 1668Manga Server เริ่มทำงานที่ http://0.0.0.0:{}", port);
    println!("📖 หน้าเว็บอ่านการ์ตูน: http://localhost:{}/", port);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
    }
}

async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => Html(content).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "static/index.html not found. Please ensure static/index.html exists.",
        )
            .into_response(),
    }
}

async fn get_manga_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cache_key = format!("manga:{}", slug.to_lowercase());
    if let Some(cached) = state.get_cached(&cache_key).await {
        return Ok(Json(cached));
    }

    match state.scraper.get_manga_detail(&slug).await {
        Ok(detail) => {
            let json_val = serde_json::to_value(&detail).unwrap_or(Value::Null);
            state.set_cached(cache_key, json_val.clone()).await;
            Ok(Json(json_val))
        }
        Err(err) => {
            let status = if err.contains("404") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(json!({ "detail": err }))))
        }
    }
}

async fn get_manga_by_query(
    State(state): State<AppState>,
    Query(query): Query<MangaQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let target = match (query.url, query.slug) {
        (Some(u), _) if !u.is_empty() => u,
        (_, Some(s)) if !s.is_empty() => s,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "detail": "กรุณาระบุพารามิเตอร์ url หรือ slug" })),
            ))
        }
    };

    let cache_key = format!("manga:{}", target.to_lowercase());
    if let Some(cached) = state.get_cached(&cache_key).await {
        return Ok(Json(cached));
    }

    match state.scraper.get_manga_detail(&target).await {
        Ok(detail) => {
            let json_val = serde_json::to_value(&detail).unwrap_or(Value::Null);
            state.set_cached(cache_key, json_val.clone()).await;
            Ok(Json(json_val))
        }
        Err(err) => {
            let status = if err.contains("404") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(json!({ "detail": err }))))
        }
    }
}

async fn search_manga(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cache_key = format!("search:{}", query.q.to_lowercase());
    if let Some(cached) = state.get_cached(&cache_key).await {
        return Ok(Json(cached));
    }

    match state.scraper.search(&query.q).await {
        Ok(results) => {
            let json_val = serde_json::to_value(&results).unwrap_or(Value::Null);
            state.set_cached(cache_key, json_val.clone()).await;
            Ok(Json(json_val))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "detail": err })),
        )),
    }
}

async fn list_latest(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1);
    let order = query.order.unwrap_or_else(|| "update".to_string());
    let cache_key = format!("list:{}:{}", page, order);

    if let Some(cached) = state.get_cached(&cache_key).await {
        return Ok(Json(cached));
    }

    match state.scraper.list_mangas(page, &order).await {
        Ok(data) => {
            let json_val = serde_json::to_value(&data).unwrap_or(Value::Null);
            state.set_cached(cache_key, json_val.clone()).await;
            Ok(Json(json_val))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "detail": err })),
        )),
    }
}

async fn get_chapter_pages(
    State(state): State<AppState>,
    Query(query): Query<ChapterQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cache_key = format!("chapter:{}", query.url.to_lowercase());
    if let Some(cached) = state.get_cached(&cache_key).await {
        return Ok(Json(cached));
    }

    match state.scraper.get_chapter_images(&query.url).await {
        Ok(data) => {
            let json_val = serde_json::to_value(&data).unwrap_or(Value::Null);
            state.set_cached(cache_key, json_val.clone()).await;
            Ok(Json(json_val))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "detail": err })),
        )),
    }
}

async fn proxy_image(
    State(state): State<AppState>,
    Query(query): Query<ProxyQuery>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    match state.scraper.fetch_image_bytes(&query.url).await {
        Ok((bytes, content_type)) => {
            let mut res = Response::new(Body::from(bytes));
            if let Ok(val) = HeaderValue::from_str(&content_type) {
                res.headers_mut().insert(header::CONTENT_TYPE, val);
            }
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"));
            Ok(res)
        }
        Err(err) => Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "detail": format!("Proxy error: {}", err) })),
        )),
    }
}

async fn debug_scrape() -> Json<Value> {
    let curl_cmd = if cfg!(windows) { "curl.exe" } else { "curl" };
    let output = tokio::process::Command::new(curl_cmd)
        .args([
            "-4",
            "--http1.1",
            "-v",
            "-s",
            "-L",
            "--max-time", "15",
            "--compressed",
            "-A", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
            "-H", "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            "-H", "Accept-Language: th-TH,th;q=0.9,en-US;q=0.8,en;q=0.7",
            "https://1668manga.com/manga/?page=1&order=update",
        ])
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Json(json!({
                "exit_status": out.status.to_string(),
                "stdout_len": stdout.len(),
                "contains_bsx": stdout.contains(".bsx"),
                "stdout_preview": stdout.chars().take(300).collect::<String>(),
                "stderr": stderr,
            }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
