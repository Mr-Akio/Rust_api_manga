---
name: api-and-clone-web
description: Extract and reverse-engineer APIs from target websites, bypass Cloudflare and WAF bot protections, and clone UI/UX design systems into modern, high-performance web applications. Use when scraping content, creating API wrappers, or cloning and redesigning web platforms.
metadata:
  version: "2.0.0"
  author: "Antigravity Pair Programmer"
---

# API Reverse-Engineering & Web Design Cloning (api-and-clone-web)

A battle-tested, end-to-end framework for reverse-engineering data APIs from protected websites, defeating Cloudflare Turnstile and WAF challenges on cloud servers, engineering compiled Rust/Node backends, and cloning/adapting UI/UX design systems into modern, responsive, mobile-first web applications.

---

## 🧭 Navigation & Specialized References

For in-depth code blueprints, technical mathematics, and concrete implementation patterns, consult the dedicated reference manuals:

- [Cloudflare & WAF Bypass Strategy](references/cloudflare-and-waf-bypass.md) — Multi-tier scraper pipeline, curl headers, Jina Reader AI relay, and anti-hotlinking proxy streams.
- [API Reverse-Engineering & Parsing](references/api-reverse-engineering.md) — DOM pattern matching, catalog selectors, chapter lists, and obfuscated JavaScript extraction.
- [Webtoon Reader Engine & Subpixel Seams](references/reader-engine-subpixel-stitching.md) — The `-1.5px` overlap mathematics, zero inline spacing, lazy loading, and auto-scroll UX.
- [UI/UX Cloning & Design Systems](references/ui-cloning-design-systems.md) — Deconstructing brand design tokens, Tailwind CSS adaptation, mobile responsiveness, and Font Awesome vector icons.
- [Rust Backend & Cloud Deployment](references/rust-backend-and-deployment.md) — Axum server architecture, Dockerfile multi-stage builds, glibc alignment, and Render/Railway setups.

---

## 🎯 When to Activate This Skill

Activate this skill when:
1. **No Public API Exists**: The target website (manga, anime, streaming, e-commerce, or portal) only provides web pages, requiring an API layer to be reverse-engineered.
2. **Cloudflare 403 Forbidden / Turnstile JS Challenge**: Datacenter IPs (AWS, Render, Railway, DigitalOcean) are challenged or blocked by Cloudflare Bot Management.
3. **UI/UX Cloning & Redesign**: The user requests a website to be cloned, redesigned, or adapted (e.g. LINE WEBTOON, Netflix, MangaDex) with a modern frontend stack.
4. **Spliced Image Continuous Readers**: Building webtoon / vertical manga reader engines that suffer from subpixel background seam fractures.
5. **Mobile-First Experience**: Requiring auto-hiding navigation bars on scroll, touch-friendly swiping, and read-status tracking.

---

## 🚦 Decision Matrix: Routing Target Workflows

```
┌────────────────────────────────────────────────────────────────────────┐
│                        TARGET INGESTION ROUTE                          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                  What is the primary requirement?
                                    │
    ┌───────────────────────────────┼───────────────────────────────┐
    ▼                               ▼                               ▼
[1. Scrape Protected API]   [2. Clone Web Design]   [3. Full-Stack End-to-End]
    │                               │                               │
    ├─► Check Cloudflare WAF        ├─► Extract Design Tokens       ├─► Phase 1: API Scrape
    ├─► Tier 1: Direct curl -4      ├─► Extract Components          ├─► Phase 2: CF Bypass
    ├─► Tier 2: Jina Relay          ├─► Tailwind CSS Adaptation     ├─► Phase 3: Rust Backend
    └─► Stream Image Proxy          └─► Mobile-First Layout         ├─► Phase 4: UI Cloning
                                                                    └─► Phase 5: Production
```

---

## 📋 Comprehensive 5-Phase Execution Runbook

### Phase 1: Target Inspection & Endpoint Reverse-Engineering

1. **Map Content Hierarchy**:
   - Catalog & Search: Determine URL query parameters (`/manga/?page=1&order=update`).
   - Details & Metadata: Identify cover poster, synopsis, author, genres, rating, and status.
   - Episode / Chapter List: Extract chapter titles, release dates, numbers, and destination URLs.
   - Media / Reader Slices: Inspect image hostnames, file extensions (`.webp`, `.avif`, `.jpg`), and hotlink protection rules.

2. **Diagnose Protection Level**:
   Execute diagnostic curl against the target endpoint:
   ```bash
   curl.exe -s -i "https://target-domain.com/catalog?page=1"
   ```
   Inspect headers:
   - `server: cloudflare` + `cf-mitigated: challenge`: Turnstile active.
   - HTTP Status `403 Forbidden` with body `<title>Just a moment...</title>`: WAF challenge triggered.
   - Read [cloudflare-and-waf-bypass.md](references/cloudflare-and-waf-bypass.md) for remediation.

---

### Phase 2: Cloudflare & Anti-Bot Bypass Architecture

Never rely on standard `reqwest` or `fetch` calls alone on datacenter servers. Implement the **Two-Tier Resilient Pipeline**:

```rust
// Tier 1: Direct IPv4 HTTP/1.1 with Realistic Browser Handshake
let direct_html = Command::new("curl")
    .arg("-s").arg("-L").arg("-4").arg("--http1.1").arg("--compressed")
    .arg("-A").arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36")
    .arg("-H").arg("Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
    .arg("-H").arg("Accept-Language: th-TH,th;q=0.9,en-US;q=0.8,en;q=0.7")
    .arg("-H").arg("Sec-CH-UA: \"Not A(Brand\";v=\"8\", \"Chromium\";v=\"132\", \"Google Chrome\";v=\"132\"")
    .arg(target_url)
    .output()?;

// Tier 2: AI Residential CDN Relay (100% Success on Render / Railway)
if direct_html.contains("403 Forbidden") || direct_html.contains("Just a moment...") {
    let jina_url = format!("https://r.jina.ai/{}", target_url);
    let relayed_html = reqwest::Client::new()
        .get(&jina_url)
        .header("X-Return-Format", "html") // Returns raw DOM HTML
        .header("User-Agent", "Mozilla/5.0 ...")
        .send().await?.text().await?;
}
```

---

### Phase 3: High-Performance Backend & Streaming Proxy

1. **Axum Asynchronous Server**:
   - Sub-millisecond response latency, single static binary, ~15MB RAM footprint.
   - Standard REST endpoints:
     - `GET /api/latest?page=1`
     - `GET /api/search?q=query`
     - `GET /api/manga/:slug`
     - `GET /api/chapter?url=...`
     - `GET /api/proxy-image?url=...`
2. **Defeating Anti-Hotlink Image Blocks**:
   Target CDNs block cross-origin image requests missing expected `Referer` headers.
   - Implement `proxy_image_handler` streaming raw image bytes with `Referer: https://target-domain.com/` and `Cache-Control: public, max-age=604800`.
   - See [cloudflare-and-waf-bypass.md#anti-hotlinking-streaming-media-proxy](references/cloudflare-and-waf-bypass.md) for full implementation.

---

### Phase 4: UI/UX Cloning & Design System Tailoring

1. **Extract and Apply Brand Tokens**:
   - Signature primary accent (e.g. Neon Green `#00dc64`).
   - Deep dark surfaces (`#0a0a0d`, `#181922`, `#2a2b38`).
   - Clean Google Font `Prompt` for seamless bilingual Thai/English readability.
   - Font Awesome 6 (`fa-solid`) vector icons replacing all inconsistent system emojis.
2. **Auto-Hiding Navigation Bar on Scroll**:
   - Track `lastScrollY`: when scrolling down, slide navbar up (`translateY(-100%)`). When scrolling up or tapping screen, slide navbar down (`translateY(0)`).
3. **Continuous Reader Engine (Zero Subpixel Seam Gaps)**:
   - Spliced webtoon images render hairline 0.5px background gaps on Retina / high-DPI displays.
   - Apply the **Zero-Seam 4-Tier CSS Formula**:
     ```css
     #readerImagesContainer { font-size: 0 !important; line-height: 0 !important; }
     .manga-page-img { display: block !important; vertical-align: top !important; transform: translateZ(0); }
     .manga-page-img + .manga-page-img { margin-top: -1.5px !important; }
     ```
   - See [reader-engine-subpixel-stitching.md](references/reader-engine-subpixel-stitching.md) for mathematical proof.
4. **Auto-Scroll to Top UX**:
   - When user triggers Next Page or Next Chapter at the bottom of the screen, immediately jump/scroll to top (`scrollY: 0`) once the new content finishes rendering.
5. **Read Status Tracking ("อ่านแล้ว ✓")**:
   - Persist completed chapters in `localStorage` by slug.
   - Render green `อ่านแล้ว ✓` badges and dimmed text in chapter lists and drawers.

---

### Phase 5: Verification, Dockerization & Cloud Deployment

1. **Headless Verification with `chrome-devtools-mcp`**:
   - Emulate mobile viewports: `viewport: "390x844x3,mobile,touch"`.
   - Check element counts, inspect network 200 OK statuses, and capture screenshot evidence.
2. **Avoid `weak version GLIBC_2.39 not found` on Render/Railway**:
   - Always pin matching Debian releases in multi-stage Docker builds:
     `FROM rust:1.83-bookworm AS builder` ──► `FROM debian:bookworm-slim`
   - See [rust-backend-and-deployment.md](references/rust-backend-and-deployment.md).

---

## 🛠️ Diagnostics & Error Recovery Ladder

When encountering issues, follow this escalation ladder:

| Observed Symptom | Underlying Cause | Immediate Remedy |
| :--- | :--- | :--- |
| `403 Forbidden` / Turnstile Challenge on Render | Cloud datacenter ASN blocked by Cloudflare | Enable Jina Reader relay (`https://r.jina.ai/<url>` with `X-Return-Format: html`). |
| Hairline lines / gaps between manga images | Subpixel rounding on fractional DPI displays | Add `margin-top: -1.5px !important;` and `font-size: 0; line-height: 0;` to container. |
| Broken images / CORS error in browser | Image CDN rejects requests lacking `Referer` | Stream images through `/api/proxy-image?url=...` with target referer. |
| `GLIBC_2.39 not found` on container startup | Docker builder image has newer glibc than runner | Pin both builder and runner stages to `debian:bookworm`. |
| Next chapter loads at bottom of screen | Viewport retains scroll position from prior chapter | Add `window.scrollTo({ top: 0, behavior: 'instant' })` in `openChapter` completion. |
| Mobile search bar squash / layout break | Fixed header lacks collapsible search toggle | Implement responsive mobile header with toggle search bar. |

---

## 📂 Deliverables Checklist

When completing a project using this skill:
- [ ] `src/scraper.rs` implements direct IPv4 HTTP/1.1 and Jina Reader Relay fallback.
- [ ] `src/handlers.rs` serves catalog, detail, chapter, and image streaming proxy.
- [ ] `static/index.html` implements cloned design system with Tailwind CSS and Font Awesome 6.
- [ ] Spliced image reader has `-1.5px` overlap and zero container line-height.
- [ ] Auto-hide navigation on scroll and auto-scroll to top on page load verified.
- [ ] `localStorage` read tracker ("อ่านแล้ว ✓") verified in chapter lists.
- [ ] Multi-stage Dockerfile verified with matching base images.
- [ ] Tested and validated locally (`localhost:8000`) and live on cloud (Render / Railway).
