use std::time::Duration;
use base64::prelude::*;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use scraper::{Html, Selector};
use serde_json::Value;

use crate::models::{
    ChapterImagesResponse, ChapterItem, MangaDetailResponse, MangaListResponse, MangaSearchResult,
};

const BASE_URL: &str = "https://1668manga.com";

#[derive(Clone)]
pub struct MangaScraper {
    client: reqwest::Client,
}

fn map_genre_to_id(genre: &str) -> String {
    let normalized = genre.trim().to_lowercase().replace("-", " ");
    let id = match normalized.as_str() {
        "action" => "3",
        "adult" => "60",
        "adventure" => "4",
        "comedy" => "5",
        "comic" => "1240",
        "demons" => "1241",
        "drama" => "6",
        "ecchi" => "56",
        "fantasy" => "7",
        "fight" => "1242",
        "game" => "1058",
        "god" => "1059",
        "harem" => "14",
        "historical" => "15",
        "horror" => "57",
        "hunters" => "1243",
        "isekai" => "778",
        "josei" => "194",
        "magic" => "1244",
        "manhua" => "384",
        "manhwa" => "299",
        "martial arts" => "12",
        "mature" => "16",
        "mecha" => "332",
        "modern" => "236",
        "mystery" => "26",
        "psychological" => "27",
        "reincarnation" => "1245",
        "revenge" => "394",
        "romance" => "17",
        "school life" => "24",
        "sci fi" | "scifi" => "18",
        "seinen" => "31",
        "shoujo" => "129",
        "shounen" => "8",
        "slice of life" => "46",
        "sports" => "124",
        "supernatural" => "9",
        "system" => "1060",
        "tragedy" => "53",
        _ => genre.trim(),
    };
    id.to_string()
}

impl MangaScraper {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
            ),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("th,en-US;q=0.9,en;q=0.8"),
        );
        headers.insert(REFERER, HeaderValue::from_static("https://1668manga.com/"));

        let client = reqwest::Client::builder()
            .http1_only()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    pub fn normalize_slug(&self, slug_or_url: &str) -> String {
        let trimmed = slug_or_url.trim();
        let decoded = urlencoding::decode(trimmed).unwrap_or(std::borrow::Cow::Borrowed(trimmed));
        let trimmed_str = decoded.trim();

        if trimmed_str.contains("1668manga.com") {
            if let Ok(parsed) = url::Url::parse(trimmed_str) {
                let segments: Vec<&str> = parsed.path_segments().map(|c| c.collect()).unwrap_or_default();
                if let Some(pos) = segments.iter().position(|&s| s == "manga") {
                    if pos + 1 < segments.len() && !segments[pos + 1].is_empty() {
                        return segments[pos + 1].to_string();
                    }
                }
                if let Some(last) = segments.iter().rev().find(|s| !s.is_empty()) {
                    return last.to_string();
                }
            }
        }

        trimmed_str.trim_matches('/').to_string()
    }

    async fn fetch_html(&self, url: &str) -> Result<String, String> {
        let curl_cmd = if cfg!(windows) { "curl.exe" } else { "curl" };

        // 1. Direct curl
        let output = tokio::process::Command::new(curl_cmd)
            .args([
                "-4",
                "--http1.1",
                "-s",
                "-L",
                "--max-time",
                "12",
                "--compressed",
                "-A",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36",
                "-H",
                "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
                "-H",
                "Accept-Language: th-TH,th;q=0.9,en-US;q=0.8,en;q=0.7",
                url,
            ])
            .output()
            .await;

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            eprintln!(
                "curl fetch {}: status={}, stdout_len={}",
                url,
                out.status,
                text.len()
            );

            if out.status.success() && !text.is_empty() {
                if !text.contains("Just a moment...")
                    && !text.contains("<title>403 Forbidden</title>")
                    && !text.contains("<title>Access denied</title>")
                {
                    return Ok(text);
                }
                eprintln!("Cloudflare bot challenge/403 detected for {}", url);
            }
        }

        // 2. Cloudflare Relay via Jina Reader (bypasses Cloudflare Datacenter block on Render/AWS)
        eprintln!("Bypassing Cloudflare via Jina Reader relay: {}", url);
        let jina_url = format!("https://r.jina.ai/{}", url);
        let jina_output = tokio::process::Command::new(curl_cmd)
            .args([
                "-s",
                "-L",
                "--max-time",
                "15",
                "-H",
                "X-Return-Format: html",
                "-A",
                "Mozilla/5.0",
                &jina_url,
            ])
            .output()
            .await;

        if let Ok(out) = jina_output {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            if out.status.success() && !text.is_empty() && !text.contains("Just a moment...") {
                eprintln!("Successfully fetched {} via Jina relay (len={})", url, text.len());
                return Ok(text);
            }
        }

        // 3. Fallback to reqwest
        eprintln!("Falling back to reqwest for {}", url);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to {}: {}", url, e))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("ไม่พบข้อมูล (404 Not Found): {}", url));
        }
        if !status.is_success() {
            return Err(format!("เกิดข้อผิดพลาดในการโหลดเว็บ ({})", status));
        }

        resp.text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))
    }

    pub async fn get_manga_detail(&self, slug_or_url: &str) -> Result<MangaDetailResponse, String> {
        let slug = self.normalize_slug(slug_or_url);
        let encoded_slug = urlencoding::encode(&slug);
        let url = format!("{}/manga/{}/", BASE_URL, encoded_slug);

        let html_content = self.fetch_html(&url).await?;
        let document = Html::parse_document(&html_content);

        // 1. Title
        let sel_title = Selector::parse("h1.entry-title").map_err(|e| e.to_string())?;
        let title = document
            .select(&sel_title)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_else(|| slug.clone());

        // 2. Alternative Title
        let sel_alt = Selector::parse(".alternative, .seriestitle").map_err(|e| e.to_string())?;
        let alternative_title = document
            .select(&sel_alt)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        // 3. Thumbnail
        let sel_thumb = Selector::parse(".thumb img").map_err(|e| e.to_string())?;
        let thumbnail = document
            .select(&sel_thumb)
            .next()
            .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
            .unwrap_or_default()
            .to_string();

        // 4. Rating
        let sel_rating = Selector::parse(".num, [itemprop='ratingValue']").map_err(|e| e.to_string())?;
        let rating = document
            .select(&sel_rating)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        // 5. Synopsis
        let sel_synopsis = Selector::parse(".entry-content[itemprop='description'], .entry-content")
            .map_err(|e| e.to_string())?;
        let synopsis = document
            .select(&sel_synopsis)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        // 6. Genres
        let sel_genre = Selector::parse(".mgen a, .genres-content a").map_err(|e| e.to_string())?;
        let genres: Vec<String> = document
            .select(&sel_genre)
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 7. Metadata (.tsinfo .imptdt)
        let sel_imptdt = Selector::parse(".tsinfo .imptdt").map_err(|e| e.to_string())?;
        let sel_inner = Selector::parse("i, a, time").map_err(|e| e.to_string())?;

        let mut status = String::new();
        let mut manga_type = String::new();
        let mut author = String::new();
        let mut artist = String::new();
        let mut released = String::new();
        let mut serialization = String::new();
        let mut updated_at = String::new();

        for item in document.select(&sel_imptdt) {
            let full_text = item.text().collect::<Vec<_>>().join(" ");
            let val = item
                .select(&sel_inner)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .unwrap_or_default();

            if full_text.contains("สถานะ") {
                status = val;
            } else if full_text.contains("ประเภท") {
                manga_type = val;
            } else if full_text.contains("ผู้เขียน") {
                author = val;
            } else if full_text.contains("Artist") {
                artist = val;
            } else if full_text.contains("Released") {
                released = val;
            } else if full_text.contains("Serialization") {
                serialization = val;
            } else if full_text.contains("อัพเดทเมื่อ") {
                updated_at = val;
            }
        }

        // 8. Chapters
        let sel_li = Selector::parse("#chapterlist li").map_err(|e| e.to_string())?;
        let sel_a = Selector::parse("a").map_err(|e| e.to_string())?;
        let sel_ch_num = Selector::parse(".chapternum").map_err(|e| e.to_string())?;
        let sel_ch_date = Selector::parse(".chapterdate").map_err(|e| e.to_string())?;
        let re_num = Regex::new(r"(\d+(?:\.\d+)?)").map_err(|e| e.to_string())?;

        let mut chapters = Vec::new();
        for li in document.select(&sel_li) {
            let mut num = li.value().attr("data-num").unwrap_or_default().to_string();
            let a_el = li.select(&sel_a).next();
            if let Some(a) = a_el {
                let c_url = a.value().attr("href").unwrap_or_default().to_string();
                let c_title = li
                    .select(&sel_ch_num)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_else(|| a.text().collect::<Vec<_>>().join(" ").trim().to_string());
                let c_date = li
                    .select(&sel_ch_date)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                if num.is_empty() {
                    if let Some(caps) = re_num.captures(&c_title) {
                        num = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    }
                }

                chapters.push(ChapterItem {
                    chapter_number: num,
                    title: c_title,
                    url: c_url,
                    date: c_date,
                });
            }
        }

        let total_chapters = chapters.len();

        Ok(MangaDetailResponse {
            title,
            alternative_title,
            slug,
            url,
            thumbnail,
            rating,
            synopsis,
            genres,
            status,
            manga_type,
            author,
            artist,
            released,
            serialization,
            updated_at,
            total_chapters,
            chapters,
        })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MangaSearchResult>, String> {
        let encoded_q = urlencoding::encode(query.trim());
        let url = format!("{}/?s={}", BASE_URL, encoded_q);

        let html_content = self.fetch_html(&url).await?;
        let document = Html::parse_document(&html_content);

        let sel_bsx = Selector::parse(".bsx").map_err(|e| e.to_string())?;
        let sel_a = Selector::parse("a").map_err(|e| e.to_string())?;
        let sel_tt = Selector::parse(".tt").map_err(|e| e.to_string())?;
        let sel_img = Selector::parse("img").map_err(|e| e.to_string())?;
        let sel_epxs = Selector::parse(".epxs").map_err(|e| e.to_string())?;
        let sel_type = Selector::parse(".type").map_err(|e| e.to_string())?;
        let sel_rating = Selector::parse(".numscore").map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        for item in document.select(&sel_bsx) {
            if let Some(a) = item.select(&sel_a).next() {
                let manga_url = a.value().attr("href").unwrap_or_default().to_string();
                if manga_url.is_empty() || seen_urls.contains(&manga_url) {
                    continue;
                }
                seen_urls.insert(manga_url.clone());

                let title = item
                    .select(&sel_tt)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_else(|| a.value().attr("title").unwrap_or_default().to_string());

                let thumbnail = item
                    .select(&sel_img)
                    .next()
                    .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
                    .unwrap_or_default()
                    .to_string();

                let latest_chapter = item
                    .select(&sel_epxs)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                let manga_type = item
                    .select(&sel_type)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                let rating = item
                    .select(&sel_rating)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                let slug = self.normalize_slug(&manga_url);

                results.push(MangaSearchResult {
                    title,
                    slug,
                    url: manga_url,
                    thumbnail,
                    latest_chapter,
                    manga_type,
                    rating,
                });
            }
        }

        Ok(results)
    }

    pub async fn get_from_feed(&self) -> Result<Vec<MangaSearchResult>, String> {
        let feed_url = format!("{}/feed/", BASE_URL);
        let xml_text = self.fetch_html(&feed_url).await?;

        let re_item = Regex::new(r"(?s)<item>(.*?)</item>").map_err(|e| e.to_string())?;
        let re_title = Regex::new(r"<title>(.*?)</title>").map_err(|e| e.to_string())?;
        let re_link = Regex::new(r"<link>(.*?)</link>").map_err(|e| e.to_string())?;
        let re_cat = Regex::new(r"<category><!\[CDATA\[(.*?)\]\]></category>").map_err(|e| e.to_string())?;
        let re_img = Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).map_err(|e| e.to_string())?;
        let re_clean = Regex::new(r"(?i)-(?:ตอนที่|chapter)-\d+(?:\.\d+)?$").map_err(|e| e.to_string())?;

        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cap in re_item.captures_iter(&xml_text) {
            let item_str = &cap[1];
            let raw_title = re_title
                .captures(item_str)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let link = re_link
                .captures(item_str)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let cat = re_cat
                .captures(item_str)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let img = re_img
                .captures(item_str)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let manga_title = if !cat.is_empty() { cat } else { raw_title.clone() };
            if seen.contains(&manga_title) {
                continue;
            }
            seen.insert(manga_title.clone());

            let mut slug = self.normalize_slug(&link);
            slug = re_clean.replace(&slug, "").to_string();

            let manga_url = format!("{}/manga/{}/", BASE_URL, slug);

            items.push(MangaSearchResult {
                title: manga_title,
                slug,
                url: manga_url,
                thumbnail: img,
                latest_chapter: raw_title,
                manga_type: "Manga".to_string(),
                rating: "9.0".to_string(),
            });
        }

        Ok(items)
    }

    pub async fn list_mangas(
        &self,
        page: u32,
        order: &str,
        manga_type: Option<&str>,
        genre: Option<&str>,
    ) -> Result<MangaListResponse, String> {
        let mut url = format!("{}/manga/?page={}&order={}", BASE_URL, page, order);
        if let Some(t) = manga_type {
            let t_clean = t.trim();
            if !t_clean.is_empty() {
                url.push_str(&format!("&type={}", urlencoding::encode(t_clean)));
            }
        }
        if let Some(g) = genre {
            let g_clean = g.trim();
            if !g_clean.is_empty() {
                let genre_id = map_genre_to_id(g_clean);
                url.push_str(&format!("&genre%5B%5D={}", urlencoding::encode(&genre_id)));
            }
        }

        let html_res = self.fetch_html(&url).await;
        let mut results = Vec::new();
        let mut has_next_page = false;

        if let Ok(html_content) = html_res {
            let document = Html::parse_document(&html_content);

            let sel_bsx = Selector::parse(".bsx").map_err(|e| e.to_string())?;
            let sel_a = Selector::parse("a").map_err(|e| e.to_string())?;
            let sel_tt = Selector::parse(".tt").map_err(|e| e.to_string())?;
            let sel_img = Selector::parse("img").map_err(|e| e.to_string())?;
            let sel_epxs = Selector::parse(".epxs").map_err(|e| e.to_string())?;
            let sel_type = Selector::parse(".type").map_err(|e| e.to_string())?;
            let sel_rating = Selector::parse(".numscore").map_err(|e| e.to_string())?;

            for item in document.select(&sel_bsx) {
                if let Some(a) = item.select(&sel_a).next() {
                    let manga_url = a.value().attr("href").unwrap_or_default().to_string();
                    let title = item
                        .select(&sel_tt)
                        .next()
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_else(|| a.value().attr("title").unwrap_or_default().to_string());

                    let thumbnail = item
                        .select(&sel_img)
                        .next()
                        .and_then(|el| el.value().attr("src").or_else(|| el.value().attr("data-src")))
                        .unwrap_or_default()
                        .to_string();

                    let latest_chapter = item
                        .select(&sel_epxs)
                        .next()
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_default();

                    let mut manga_type_val = item
                        .select(&sel_type)
                        .next()
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_default();

                    if manga_type_val.is_empty() {
                        if let Some(t) = manga_type {
                            let t_clean = t.trim();
                            if !t_clean.is_empty() {
                                let mut c = t_clean.chars();
                                manga_type_val = match c.next() {
                                    None => String::new(),
                                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                };
                            }
                        }
                    }

                    let rating = item
                        .select(&sel_rating)
                        .next()
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_default();

                    let slug = self.normalize_slug(&manga_url);

                    results.push(MangaSearchResult {
                        title,
                        slug,
                        url: manga_url,
                        thumbnail,
                        latest_chapter,
                        manga_type: manga_type_val,
                        rating,
                    });
                }
            }

            if let Ok(sel_next) = Selector::parse(".r, a.next") {
                has_next_page = document.select(&sel_next).next().is_some();
            }
        }

        // If results empty on first page and no filters applied, fall back to RSS feed
        if results.is_empty() && page == 1 && manga_type.is_none() && genre.is_none() {
            if let Ok(feed_items) = self.get_from_feed().await {
                if !feed_items.is_empty() {
                    let count = feed_items.len();
                    return Ok(MangaListResponse {
                        page,
                        order: order.to_string(),
                        count,
                        has_next_page: false,
                        results: feed_items,
                    });
                }
            }
        }

        if results.is_empty() && page > 1 {
            return Err("ไม่สามารถโหลดรายการมังงะได้".to_string());
        }

        let count = results.len();
        Ok(MangaListResponse {
            page,
            order: order.to_string(),
            count,
            has_next_page,
            results,
        })
    }

    pub async fn get_chapter_images(&self, chapter_url_or_slug: &str) -> Result<ChapterImagesResponse, String> {
        let mut url = chapter_url_or_slug.trim().to_string();
        if !url.starts_with("http") {
            url = format!("{}/{}/", BASE_URL, url.trim_matches('/'));
        }

        let html_content = self.fetch_html(&url).await?;
        let document = Html::parse_document(&html_content);

        // 1. Search for base64 ts_reader script
        let sel_script = Selector::parse("script[src]").map_err(|e| e.to_string())?;
        let re_ts_reader = Regex::new(r"ts_reader\.run\((.*?)\);").map_err(|e| e.to_string())?;
        let mut images = Vec::new();

        for script in document.select(&sel_script) {
            if let Some(src) = script.value().attr("src") {
                if let Some(b64_part) = src.strip_prefix("data:text/javascript;base64,") {
                    if let Ok(decoded_bytes) = BASE64_STANDARD.decode(b64_part) {
                        let decoded_str = String::from_utf8_lossy(&decoded_bytes);
                        if decoded_str.contains("ts_reader.run") {
                            if let Some(caps) = re_ts_reader.captures(&decoded_str) {
                                if let Some(json_str) = caps.get(1) {
                                    if let Ok(val) = serde_json::from_str::<Value>(json_str.as_str()) {
                                        if let Some(sources) = val.get("sources").and_then(|s| s.as_array()) {
                                            if let Some(first_source) = sources.first() {
                                                if let Some(imgs) = first_source.get("images").and_then(|i| i.as_array()) {
                                                    for img in imgs {
                                                        if let Some(img_url) = img.as_str() {
                                                            images.push(img_url.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if !images.is_empty() {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback to #readerarea img
        if images.is_empty() {
            let sel_reader_img = Selector::parse("#readerarea img").map_err(|e| e.to_string())?;
            for img in document.select(&sel_reader_img) {
                if let Some(src) = img.value().attr("src").or_else(|| img.value().attr("data-src")) {
                    images.push(src.to_string());
                }
            }
        }

        // Title
        let sel_ch_title = Selector::parse("h1.entry-title").map_err(|e| e.to_string())?;
        let title = document
            .select(&sel_ch_title)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .unwrap_or_default();

        let total_images = images.len();

        Ok(ChapterImagesResponse {
            title,
            url,
            total_images,
            images,
        })
    }

    pub async fn fetch_image_bytes(&self, url: &str) -> Result<(bytes::Bytes, String), String> {
        let resp = self
            .client
            .get(url)
            .header(REFERER, "")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch image {}: {}", url, e))?;

        if !resp.status().is_success() {
            return Err(format!("Image fetch status: {}", resp.status()));
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read image bytes: {}", e))?;

        Ok((bytes, content_type))
    }
}
