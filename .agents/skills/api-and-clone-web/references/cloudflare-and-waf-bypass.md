# Cloudflare & WAF Bypass Blueprint

This document details the exact bypass mechanisms used when target sites protect their content behind Cloudflare WAF, Turnstile challenges, and IP ASN reputation filters.

---

## 1. The Threat Model & Detection Mechanisms

Target websites frequently employ Cloudflare's Bot Management, which checks:
1. **IP ASN Reputation**: Datacenter IP ranges (AWS, Google Cloud, Render, Railway, DigitalOcean, Linode) are flagged as non-human bots.
2. **TLS / JA3 Fingerprint**: Go, Python requests, or basic curl with default TLS handshakes are flagged immediately.
3. **HTTP Protocol Version**: Automated crawlers often fail on HTTP/2 multiplexing or send broken ALPN headers.
4. **Browser Headers**: Missing `Sec-CH-UA`, `Sec-Fetch-*`, `Accept-Language`, or compressed encoding headers trigger automatic blocks.

When triggered, Cloudflare responds with:
- HTTP Status: `403 Forbidden`
- Response Header: `cf-mitigated: challenge`
- HTML Body: `<title>Just a moment...</title>` containing Turnstile JavaScript challenges.

---

## 2. Multi-Tier Resolution Strategy

To achieve 100% uptime without manual intervention:

```
[Request Initiation]
         │
         ▼
[Tier 1: Direct IPv4 HTTP/1.1 with Browser Fingerprint]
         │
         ├─► HTTP 200 OK & valid HTML ──► Success
         │
         ▼ Returns 403 Forbidden / "Just a moment..."
[Tier 2: Jina Reader HTML Relay (Residential CDN Fallback)]
         │
         ├─► HTTP 200 OK & parsed HTML ──► Success
         │
         ▼ Network Failure / Rate Limit
[Tier 3: Stream Proxy / Alternative Mirror]
```

---

## 3. Tier 1: Direct Curl Execution with Browser Fingerprint

When running on local machine (residential IP) or unflagged servers, a properly tuned `curl` command passes Cloudflare with high success.

### Key Arguments:
- `-4`: Force IPv4. Broken datacenter IPv6 routes frequently trigger WAF drop rules.
- `--http1.1`: Avoid HTTP/2 stream multiplexing anomalies that trip bot heuristics.
- `-s -L`: Follow redirects silently up to max 10.
- `--compressed`: Request `gzip, deflate, br, zstd` like a genuine browser.

### Rust Implementation:
```rust
use std::process::Command;

pub fn fetch_with_curl(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .arg("-s")
        .arg("-L")
        .arg("-4")
        .arg("--http1.1")
        .arg("--compressed")
        .arg("-A").arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
        .arg("-H").arg("Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .arg("-H").arg("Accept-Language: th-TH,th;q=0.9,en-US;q=0.8,en;q=0.7")
        .arg("-H").arg("Sec-CH-UA: \"Not A(Brand\";v=\"8\", \"Chromium\";v=\"132\", \"Google Chrome\";v=\"132\"")
        .arg("-H").arg("Sec-CH-UA-Mobile: ?0")
        .arg("-H").arg("Sec-CH-UA-Platform: \"Windows\"")
        .arg("-H").arg("Sec-Fetch-Dest: document")
        .arg("-H").arg("Sec-Fetch-Mode: navigate")
        .arg("-H").arg("Sec-Fetch-Site: none")
        .arg("-H").arg("Sec-Fetch-User: ?1")
        .arg(url)
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        return Err(format!("curl exited with code {:?}", output.status.code()));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 from curl: {}", e))
}
```

---

## 4. Tier 2: Jina Reader HTML Relay (100% Turnstile Bypass)

When deployed on cloud platforms (e.g. Render, Railway, AWS ECS), the server's IP address will be unconditionally challenged by Turnstile. No amount of header spoofing can bypass an ASN-level IP block.

### The Mechanism:
- **Jina Reader** (`r.jina.ai`) operates distributed residential proxies with real headless browser pools.
- Endpoint: `https://r.jina.ai/<full_target_url>`
- Crucial Header: `X-Return-Format: html`
  - *Note*: By default, Jina Reader converts web pages to Markdown. Passing `X-Return-Format: html` instructs Jina to return the raw, fully rendered DOM HTML with all scripts and images resolved.

### Rust Implementation:
```rust
pub async fn fetch_page_robust(url: &str) -> Result<String, String> {
    // 1. Try Tier 1: Direct IPv4 HTTP/1.1
    let direct_html = fetch_with_curl(url).unwrap_or_default();

    // Check if Cloudflare challenged or blocked
    let is_blocked = direct_html.contains("403 Forbidden") 
        || direct_html.contains("Just a moment...")
        || direct_html.contains("cf-mitigated")
        || direct_html.len() < 500;

    if !is_blocked && !direct_html.is_empty() {
        return Ok(direct_html);
    }

    // 2. Fallback to Tier 2: Jina Reader HTML Relay
    eprintln!("[Scraper] Direct request challenged (403). Relaying through Jina Reader...");
    let jina_url = format!("https://r.jina.ai/{}", url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&jina_url)
        .header("X-Return-Format", "html")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Jina Relay request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Jina Relay returned status: {}", resp.status()));
    }

    let relayed_html = resp.text().await.map_err(|e| e.to_string())?;
    Ok(relayed_html)
}
```

---

## 5. Anti-Hotlinking: Streaming Media Proxy

Target CDNs (e.g. `cdn.1668manga.com`, `img.168toon.com`) inspect the `Referer` header of incoming image requests. If the image is loaded from our domain, the CDN returns `403 Forbidden`.

### The Solution:
Create a dedicated proxy endpoint `/api/proxy-image?url=...` that downloads and streams the media bytes while providing the target's expected `Referer`:

```rust
use axum::{extract::Query, response::{IntoResponse, Response}, http::StatusCode, body::Body};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProxyQuery {
    pub url: String,
}

pub async fn proxy_image_handler(Query(params): Query<ProxyQuery>) -> Response {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap();

    let resp = match client.get(&params.url)
        .header("Referer", "https://1668manga.com/")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await {
            Ok(r) if r.status().is_success() => r,
            _ => return StatusCode::BAD_GATEWAY.into_response(),
        };

    let content_type = resp.headers().get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let body = Body::from_stream(resp.bytes_stream());

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Cache-Control", "public, max-age=604800, immutable")
        .header("Access-Control-Allow-Origin", "*")
        .body(body)
        .unwrap()
}
```

### Frontend Auto-Fallback:
In the frontend viewer, images attempt direct loading first (fastest, no server bandwidth used). If the CDN blocks with CORS or 403, the `onerror` event seamlessly switches to the proxy stream:
```javascript
img.src = originalImageUrl;
img.onerror = () => {
    img.onerror = null; // Prevent infinite loop
    img.src = `/api/proxy-image?url=${encodeURIComponent(originalImageUrl)}`;
};
```

