# API Reverse-Engineering & HTML Data Extraction

This document describes how to reverse-engineer web targets lacking public APIs and extract structured JSON data using modern CSS selectors and DOM parsing.

---

## 1. Catalog & Latest Updates Parsing

Manga and comic sites (e.g. MangaStream / Madara WordPress themes) structure their catalog in card grids:

### Typical DOM Pattern:
```html
<div class="bsx">
    <a href="https://1668manga.com/manga/magic-emperor/" title="Magic Emperor">
        <div class="limit">
            <img src="https://cdn.../cover.webp" class="ts-post-image">
            <span class="type">Manhua</span>
        </div>
        <div class="bigor">
            <div class="tt">Magic Emperor</div>
            <div class="adds">
                <div class="epxs">ตอนที่ 934</div>
                <div class="rating"><i class="num">8.0</i></div>
            </div>
        </div>
    </a>
</div>
```

### Rust Scraper Selector Extraction:
```rust
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MangaItem {
    pub title: String,
    pub slug: String,
    pub url: String,
    pub thumbnail: String,
    pub latest_chapter: String,
    pub rating: String,
    pub manga_type: String,
}

pub fn parse_catalog_page(html_content: &str) -> Vec<MangaItem> {
    let document = Html::parse_document(html_content);
    let card_sel = Selector::parse(".bsx, .animposx, .page-item-detail").unwrap();
    let title_sel = Selector::parse(".tt, .title, h4, h3").unwrap();
    let link_sel = Selector::parse("a").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let ep_sel = Selector::parse(".epxs, .chapter, .font-meta").unwrap();
    let rating_sel = Selector::parse(".num, .rating, .score").unwrap();

    let mut items = Vec::new();

    for card in document.select(&card_sel) {
        let link = card.select(&link_sel).next();
        let url = link.and_then(|a| a.value().attr("href")).unwrap_or("").to_string();
        let title = card.select(&title_sel).next()
            .map(|t| t.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| link.and_then(|a| a.value().attr("title")).unwrap_or("").to_string());

        let thumbnail = card.select(&img_sel).next()
            .and_then(|img| {
                img.value().attr("data-src")
                    .or_else(|| img.value().attr("data-lazy-src"))
                    .or_else(|| img.value().attr("src"))
            })
            .unwrap_or("")
            .to_string();

        let latest_chapter = card.select(&ep_sel).next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let rating = card.select(&rating_sel).next()
            .map(|r| r.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let slug = url.trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("")
            .to_string();

        if !title.is_empty() && !url.is_empty() {
            items.push(MangaItem {
                title,
                slug,
                url,
                thumbnail,
                latest_chapter,
                rating,
                manga_type: "Manga".to_string(),
            });
        }
    }

    items
}
```

---

## 2. Manga Details & Chapter List Extraction

### Typical DOM Pattern:
```html
<h1 class="entry-title">Magic Emperor</h1>
<div class="entry-content-single">
    <p>เรื่องย่อของมังงะ...</p>
</div>
<ul class="clstyle">
    <li data-num="934">
        <a href="https://1668manga.com/magic-emperor-ตอนที่-934/">
            <span class="chapternum">ตอนที่ 934</span>
            <span class="chapterdate">November 13, 2026</span>
        </a>
    </li>
</ul>
```

### Extraction Logic:
Extract synopsis, author, alternative titles, genres, and sort the chapter array:
```rust
#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub chapter_number: String,
    pub url: String,
    pub date: String,
}

#[derive(Serialize, Deserialize)]
pub struct MangaDetail {
    pub title: String,
    pub alternative_title: String,
    pub synopsis: String,
    pub thumbnail: String,
    pub author: String,
    pub artist: String,
    pub status: String,
    pub genres: Vec<String>,
    pub chapters: Vec<Chapter>,
}
```

---

## 3. Chapter Image Extractor & Obfuscation Bypassing

Webtoon reader pages load images in one of three ways:
1. **Static HTML Image Elements**:
   `<div id="readerarea"><img src="https://img.cdn.../01.jpg"></div>`
2. **Inline JSON Variables**:
   `<script>var ts_reader = { "sources": [{ "images": [...] }] };</script>`
3. **Obfuscated / Packed JavaScript (`eval(function(p,a,c,k,e,d)...`)**:
   Regex pattern matching can extract raw image URLs from script blocks without running full JS VMs.

### Regex Extraction Fallback for Obfuscated Image Scripts:
```rust
use regex::Regex;

pub fn extract_chapter_images(html_content: &str) -> Vec<String> {
    let document = Html::parse_document(html_content);
    let reader_sel = Selector::parse("#readerarea img, .reading-content img, .reader-area img").unwrap();

    let mut images = Vec::new();

    // 1. Direct <img> tags
    for img in document.select(&reader_sel) {
        if let Some(src) = img.value().attr("src")
            .or_else(|| img.value().attr("data-src"))
            .or_else(|| img.value().attr("data-lazy-src")) {
            let clean = src.trim().to_string();
            if !clean.is_empty() && (clean.starts_with("http://") || clean.starts_with("https://")) {
                images.push(clean);
            }
        }
    }

    // 2. If 0 images found, search inline JSON blocks
    if images.is_empty() {
        let re = Regex::new(r#""(https?://[^"\\]+?\.(?:jpe?g|png|webp|avif)[^"\\]*)""#).unwrap();
        for cap in re.captures_iter(html_content) {
            let url = cap[1].replace("\\/", "/");
            if !images.contains(&url) {
                images.push(url);
            }
        }
    }

    images
}
```

