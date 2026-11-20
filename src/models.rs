use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterItem {
    pub chapter_number: String,
    pub title: String,
    pub url: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDetailResponse {
    pub title: String,
    pub alternative_title: String,
    pub slug: String,
    pub url: String,
    pub thumbnail: String,
    pub rating: String,
    pub synopsis: String,
    pub genres: Vec<String>,
    pub status: String,
    #[serde(rename = "type")]
    pub manga_type: String,
    pub author: String,
    pub artist: String,
    pub released: String,
    pub serialization: String,
    pub updated_at: String,
    pub total_chapters: usize,
    pub chapters: Vec<ChapterItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaSearchResult {
    pub title: String,
    pub slug: String,
    pub url: String,
    pub thumbnail: String,
    pub latest_chapter: String,
    #[serde(rename = "type")]
    pub manga_type: String,
    pub rating: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaListResponse {
    pub page: u32,
    pub order: String,
    pub count: usize,
    pub has_next_page: bool,
    pub results: Vec<MangaSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterImagesResponse {
    pub title: String,
    pub url: String,
    pub total_images: usize,
    pub images: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MangaQuery {
    pub url: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub order: Option<String>,
    #[serde(rename = "type")]
    pub manga_type: Option<String>,
    pub genre: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterQuery {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxyQuery {
    pub url: String,
}

