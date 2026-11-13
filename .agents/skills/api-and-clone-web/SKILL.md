---
name: api-and-clone-web
description: Extract and reverse-engineer APIs from target websites, bypass Cloudflare and WAF bot protections, and clone UI/UX design systems into modern, high-performance web applications. Use when scraping content, creating API wrappers, or cloning and redesigning web platforms.
metadata:
  version: "1.0.0"
  author: "Antigravity Pair Programmer"
---

# API & Web Clone Skill (api-and-clone-web)

A comprehensive, field-tested skill for reverse-engineering web content, bypassing Cloudflare/WAF bot protections, building compiled backend APIs, and reproducing/adapting UI/UX design systems into modern, responsive web applications.

---

## 🎯 When to Use This Skill

Activate this skill when:
1. **API Extraction / Reverse-Engineering**: The user wants to scrape or build an API from a website that has no public API or protects its content.
2. **Cloudflare / WAF Bypass**: Target websites return `403 Forbidden`, `503 Service Unavailable`, or Cloudflare Turnstile JavaScript challenges on cloud servers (Render, Railway, AWS).
3. **Web Design Cloning & Adaptation**: The user wants to clone a well-known site's UI (e.g. LINE WEBTOON, Netflix, MangaDex) and hook it up to custom or scraped backend APIs.
4. **Spliced Media & Reader Engines**: Building image/comic readers that require zero-gap, continuous vertical scrolling without subpixel rendering fractures.

---

## 🏗️ The 5-Phase Workflow

```
[Phase 1: Target Inspection] ──► [Phase 2: Cloudflare & Anti-Bot Bypass]
                                                   │
                                                   ▼
[Phase 4: UI/UX Cloning & Reader] ◄── [Phase 3: High-Performance Backend]
            │
            ▼
[Phase 5: Responsive Verification & Deployment]
```

---

## Phase 1: Target Inspection & Endpoint Reverse-Engineering

### 1. Identify Content Hierarchy
- **Catalog / Latest Updates**: Pagination structure (e.g. `?page=1&order=update`).
- **Item Details**: Slug, title, synopsis, status, tags/genres, cover artwork.
- **Episodes / Chapters**: List of episodes, release dates, chapter numbers, content URLs.
- **Content Pages / Media**: Image asset URLs, CDN hostnames, anti-hotlinking headers.

### 2. Network Traffic Inspection
Use `chrome-devtools-mcp` or `curl.exe` to inspect target network responses:
```bash
curl.exe -s -i "https://target-site.com/catalog?page=1"
```
Look for:
- Response status code (`200 OK` vs `403 Forbidden`).
- Response headers: `Server: cloudflare`, `Cf-Mitigated: challenge`, `strict-transport-security`.
- Media CDN headers: `referrer-policy`, `access-control-allow-origin`.

---

## Phase 2: Cloudflare & WAF Bypass Strategies

Cloud platforms (Render, Railway, AWS, DigitalOcean) share datacenter IP ranges (ASNs) heavily flagged by Cloudflare WAF. Standard HTTP client calls fail with `403 Forbidden` (`Just a moment...` Turnstile challenge).

### The Multi-Tier Scraper Pipeline:

```
[Incoming Request]
        │
        ▼
[Tier 1: Direct IPv4 HTTP/1.1 with Realistic Headers]
        │
        ├─► Success (200 OK) ──► Parse HTML
        │
        ▼ 403 Forbidden / Challenge
[Tier 2: Jina Reader HTML Relay (AI CDN Fallback)]
        │
        ├─► Success (200 OK) ──► Parse HTML
        │
        ▼
[Tier 3: Streaming Proxy / Cache]
```

### 1. Direct Curl Command Configuration (Tier 1)
Run with HTTP/1.1, IPv4 forced, and complete modern browser fingerprint:
```rust
let output = Command::new("curl")
    .arg("-s")
    .arg("-L")
    .arg("-4")              // Force IPv4 to avoid broken IPv6 routing
    .arg("--http1.1")       // HTTP/1.1 mimics typical browser fallbacks
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
    .arg(target_url)
    .output()?;
```

### 2. Jina Reader Relay (Tier 2 - The 100% Cloudflare Bypass)
When datacenter IP is hard-blocked by Turnstile JS challenge:
- Prepend `https://r.jina.ai/<target_url>`
- Send header `X-Return-Format: html`
- Jina executes browser automation across global residential relays, fully resolving JavaScript Turnstile gates and returning the parsed rendered HTML.

```rust
if html.contains("403 Forbidden") || html.contains("Just a moment...") {
    let jina_url = format!("https://r.jina.ai/{}", target_url);
    let jina_html = reqwest::Client::new()
        .get(&jina_url)
        .header("X-Return-Format", "html")
        .header("User-Agent", "Mozilla/5.0 ...")
        .send()
        .await?
        .text()
        .await?;
}
```

---

## Phase 3: Backend API & Media Proxy

### 1. Architecture (Rust Axum / Actix or Node/Python)
- **High-speed static compilation**: Rust with Axum yields sub-millisecond response times, low RAM usage (~15MB), and easy single-binary distribution.
- **CORS Handling**: Allow cross-origin requests (`tower_http::cors::CorsLayer`).
- **Interactive OpenAPI Docs**: Integrate Swagger UI via `utoipa-swagger-ui` at `/docs`.

### 2. Defeating Hotlink Protection (Image Proxy Stream)
Target image CDNs frequently verify `Referer: <target-domain>`. Loading images directly in third-party browsers causes `403 Forbidden` broken images.
Implement a streaming proxy endpoint:
```rust
// GET /api/proxy-image?url=https://img.cdn.com/page1.jpg
pub async fn proxy_image(Query(params): Query<ProxyQuery>) -> Response {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    let res = client.get(&params.url)
        .header("Referer", "https://target-domain.com/")
        .header("User-Agent", "Mozilla/5.0 ...")
        .send()
        .await;

    // Stream image bytes directly with proper Content-Type & Cache-Control
    match res {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp.headers().get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("image/jpeg")
                .to_string();
            let body = Body::from_stream(resp.bytes_stream());
            Response::builder()
                .status(status)
                .header("Content-Type", content_type)
                .header("Cache-Control", "public, max-age=86400")
                .body(body)
                .unwrap()
        }
        Err(_) => StatusCode::BAD_GATEWAY.into_response(),
    }
}
```

---

## Phase 4: UI/UX Cloning & Design System Tailoring

### 1. Extract Design Tokens
- **Palette**: Signature brand colors (e.g. Neon Green `#00dc64`, Deep Black `#0a0a0d`, Card `#181922`, Border `#2a2b38`).
- **Typography**: Prompt font (`Prompt, sans-serif`) for modern Thai/English reading.
- **Icons**: Use Font Awesome 6 (`fa-solid`) over raw emojis for sharp vector rendering across devices.

### 2. Auto-Hiding Navbar on Scroll (UX Industry Standard)
Hides navigation when scrolling down to maximize viewport; reveals instantly when scrolling up or tapping screen:
```javascript
let lastScrollY = window.scrollY;
window.addEventListener('scroll', () => {
    const currentY = window.scrollY;
    const isScrollingDown = currentY > lastScrollY && currentY > 60;
    
    if (isScrollingDown) {
        navbar.style.transform = 'translateY(-100%)';
    } else {
        navbar.style.transform = 'translateY(0)';
    }
    lastScrollY = currentY;
}, { passive: true });
```

### 3. Spliced Continuous Strip (Zero Seam Gap Solution)
Webtoon image slices sliced every 1,000–1,500px exhibit hairline background seams on Retina / fractional scaling displays (125%, 150%, 200% DPI).
**The Definite CSS Solution**:
```css
#readerImagesContainer {
    font-size: 0 !important;
    line-height: 0 !important;
    letter-spacing: 0 !important;
    word-spacing: 0 !important;
    padding: 0 !important;
}

.manga-page-img {
    display: block !important;
    width: 100% !important;
    max-width: 800px !important;
    margin-left: auto !important;
    margin-right: auto !important;
    padding: 0 !important;
    border: 0 !important;
    outline: none !important;
    vertical-align: top !important;
    user-select: none;
    -webkit-user-drag: none;
    image-rendering: -webkit-optimize-contrast;
    transform: translateZ(0); /* Hardware acceleration */
}

/* 1.5px subpixel overlap eliminates all fractional rendering seams */
.manga-page-img + .manga-page-img,
.manga-page-img:not(:first-child) {
    margin-top: -1.5px !important;
}
```

### 4. Read Status Tracking ("อ่านแล้ว ✓")
Store reading history and completed chapters in `localStorage`:
```javascript
function markChapterAsRead(slug, chapterUrl) {
    const store = JSON.parse(localStorage.getItem('read_chapters') || '{}');
    if (!store[slug]) store[slug] = [];
    if (!store[slug].includes(chapterUrl)) {
        store[slug].push(chapterUrl);
        localStorage.setItem('read_chapters', JSON.stringify(store));
    }
}
```

### 5. Auto Scroll-to-Top upon Page/Chapter Load
When user clicks "Next Page" or "Next Chapter" at the bottom of a chapter, auto-scroll to the top immediately once new items render:
```javascript
async function changePage(delta) {
    const next = currentPage + delta;
    if (next >= 1) {
        await loadLatestManga(next);
        const section = document.getElementById('allMangaSection');
        if (section) {
            const top = section.getBoundingClientRect().top + window.pageYOffset - 75;
            window.scrollTo({ top: Math.max(0, top), behavior: 'smooth' });
        }
    }
}
```

---

## Phase 5: Verification & Deployment

### 1. Local Headless Verification
Use `chrome-devtools-mcp` to emulate mobile viewports (`390x844x3,mobile,touch`), verify DOM counts, test network calls, and capture screenshots.

### 2. Multi-Stage Dockerfile (glibc / Alpine Compatibility)
To prevent `weak version 'GLIBC_2.39' not found` errors on Render/Railway, build in the same base image as the final runtime:
```dockerfile
# Builder Stage
FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY static ./static
RUN cargo build --release

# Runner Stage
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/manga_api /app/manga_api
COPY static /app/static
ENV PORT=8000
EXPOSE 8000
CMD ["./manga_api"]
```

---

## 📋 Checklist for Completed Projects

- [ ] Target endpoints and pagination tested via direct curl & Jina relay.
- [ ] Cloudflare Turnstile fallback verified for datacenter IPs.
- [ ] Streaming image proxy active with anti-hotlinking referer override.
- [ ] Responsive UI verified on mobile (390px) and desktop (1280px).
- [ ] Font Awesome icons replace all raw emojis.
- [ ] Navbar auto-hides on scroll down and reveals on scroll up.
- [ ] Spliced images overlap by `-1.5px` with zero hairline gaps.
- [ ] Read status badges ("อ่านแล้ว ✓") persist in `localStorage`.
- [ ] Next Page / Next Chapter auto-scrolls to top after content loads.
- [ ] Dockerfile glibc runtime verified for cloud deployments (Render / Railway).
