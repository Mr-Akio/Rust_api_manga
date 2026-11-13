import re
import base64
import json
import urllib.parse
from typing import Optional, List, Dict, Any
import requests
from bs4 import BeautifulSoup

BASE_URL = "https://1668manga.com"

DEFAULT_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
    "Accept-Language": "th,en-US;q=0.9,en;q=0.8",
    "Referer": BASE_URL + "/",
}


class MangaScraper:
    def __init__(self):
        self.session = requests.Session()
        self.session.headers.update(DEFAULT_HEADERS)

    def _fetch_soup(self, url: str) -> BeautifulSoup:
        try:
            resp = self.session.get(url, timeout=15)
            resp.raise_for_status()
            return BeautifulSoup(resp.text, "html.parser")
        except requests.HTTPError as e:
            if resp.status_code == 404:
                raise ValueError(f"ไม่พบข้อมูล (404 Not Found): {url}")
            raise RuntimeError(f"เกิดข้อผิดพลาดในการโหลดเว็บ ({resp.status_code}): {e}")
        except Exception as e:
            raise RuntimeError(f"ไม่สามารถเชื่อมต่อกับ {url} ได้: {str(e)}")

    def _normalize_slug(self, slug_or_url: str) -> str:
        """Extract clean slug from either a slug or full URL."""
        slug_or_url = urllib.parse.unquote(slug_or_url.strip())
        if "1668manga.com" in slug_or_url:
            parsed = urllib.parse.urlparse(slug_or_url)
            path_parts = [p for p in parsed.path.split("/") if p]
            if "manga" in path_parts:
                idx = path_parts.index("manga")
                if len(path_parts) > idx + 1:
                    return path_parts[idx + 1]
            if path_parts:
                return path_parts[-1]
        # Remove any leading/trailing slashes
        return slug_or_url.strip("/")

    def get_manga_detail(self, slug_or_url: str) -> Dict[str, Any]:
        """
        ดึงข้อมูลชื่อเรื่อง และทุกตอนของมังงะ
        """
        slug = self._normalize_slug(slug_or_url)
        url = f"{BASE_URL}/manga/{urllib.parse.quote(slug)}/"
        soup = self._fetch_soup(url)

        # 1. ชื่อเรื่องหลัก (Title)
        title_elem = soup.find("h1", class_="entry-title")
        title = title_elem.text.strip() if title_elem else ""
        if not title:
            # Fallback title from meta og:title
            og_title = soup.find("meta", property="og:title")
            title = og_title.get("content", "").strip() if og_title else slug

        # 2. ชื่อเรื่องทางเลือก (Alternative titles)
        alt_elem = soup.select_one(".alternative, .seriestitle")
        alt_title = alt_elem.text.strip() if alt_elem else ""

        # 3. รูปปก (Thumbnail)
        thumb_elem = soup.select_one(".thumb img")
        thumbnail = ""
        if thumb_elem:
            thumbnail = thumb_elem.get("src") or thumb_elem.get("data-src") or ""

        # 4. เรื่องย่อ (Synopsis)
        synopsis_elem = soup.select_one('.entry-content[itemprop="description"], .entry-content')
        synopsis = ""
        if synopsis_elem:
            synopsis = synopsis_elem.text.strip()

        # 5. หมวดหมู่ (Genres)
        genres = [a.text.strip() for a in soup.select(".mgen a, .genres-content a")]

        # 6. คะแนน (Rating)
        rating_elem = soup.select_one(".num, [itemprop='ratingValue']")
        rating = rating_elem.text.strip() if rating_elem else ""

        # 7. ข้อมูลเพิ่มเติม (.tsinfo .imptdt)
        meta_info: Dict[str, str] = {}
        for imptdt in soup.select(".tsinfo .imptdt"):
            raw_text = imptdt.get_text(separator=" ", strip=True)
            # Find label and value
            val_elem = imptdt.find(["i", "a", "time"])
            if val_elem:
                val = val_elem.text.strip()
                # Remove val from raw_text to get label
                label = raw_text.replace(val, "").strip()
                if label:
                    meta_info[label] = val

        # 8. รายการตอนทั้งหมด (Chapters list)
        chapters: List[Dict[str, Any]] = []
        for li in soup.select("#chapterlist li"):
            chapter_num = li.get("data-num", "")
            a_elem = li.select_one("a")
            title_elem = li.select_one(".chapternum")
            date_elem = li.select_one(".chapterdate")

            if a_elem:
                c_url = a_elem.get("href", "")
                c_title = title_elem.text.strip() if title_elem else a_elem.text.strip()
                c_date = date_elem.text.strip() if date_elem else ""

                if not chapter_num:
                    # Try to extract number from title
                    match = re.search(r"(\d+(?:\.\d+)?)", c_title)
                    chapter_num = match.group(1) if match else ""

                chapters.append({
                    "chapter_number": chapter_num,
                    "title": c_title,
                    "url": c_url,
                    "date": c_date
                })

        return {
            "title": title,
            "alternative_title": alt_title,
            "slug": slug,
            "url": url,
            "thumbnail": thumbnail,
            "rating": rating,
            "synopsis": synopsis,
            "genres": genres,
            "status": meta_info.get("สถานะ", ""),
            "type": meta_info.get("ประเภท", ""),
            "author": meta_info.get("ผู้เขียน", ""),
            "artist": meta_info.get("Artist", ""),
            "released": meta_info.get("Released", ""),
            "serialization": meta_info.get("Serialization", ""),
            "updated_at": meta_info.get("อัพเดทเมื่อ", ""),
            "total_chapters": len(chapters),
            "chapters": chapters
        }

    def search(self, query: str) -> List[Dict[str, Any]]:
        """ค้นหามังงะตามคำค้นหา"""
        search_url = f"{BASE_URL}/?s={urllib.parse.quote(query)}"
        soup = self._fetch_soup(search_url)

        results = []
        seen_urls = set()

        for item in soup.select(".bsx"):
            a = item.select_one("a")
            if not a:
                continue
            manga_url = a.get("href", "")
            if not manga_url or manga_url in seen_urls:
                continue
            seen_urls.add(manga_url)

            title_elem = item.select_one(".tt")
            title = title_elem.text.strip() if title_elem else a.get("title", "")

            img_elem = item.select_one("img")
            thumb = ""
            if img_elem:
                thumb = img_elem.get("src") or img_elem.get("data-src") or ""

            latest_ch_elem = item.select_one(".epxs")
            latest_chapter = latest_ch_elem.text.strip() if latest_ch_elem else ""

            type_elem = item.select_one(".type")
            manga_type = type_elem.text.strip() if type_elem else ""

            rating_elem = item.select_one(".numscore")
            rating = rating_elem.text.strip() if rating_elem else ""

            slug = self._normalize_slug(manga_url)

            results.append({
                "title": title,
                "slug": slug,
                "url": manga_url,
                "thumbnail": thumb,
                "latest_chapter": latest_chapter,
                "type": manga_type,
                "rating": rating
            })

        return results

    def list_mangas(self, page: int = 1, order: str = "update") -> Dict[str, Any]:
        """ดึงรายการมังงะตามหน้าและการเรียงลำดับ"""
        catalog_url = f"{BASE_URL}/manga/?page={page}&order={order}"
        soup = self._fetch_soup(catalog_url)

        items = []
        for item in soup.select(".bsx"):
            a = item.select_one("a")
            if not a:
                continue
            manga_url = a.get("href", "")
            title_elem = item.select_one(".tt")
            title = title_elem.text.strip() if title_elem else a.get("title", "")

            img_elem = item.select_one("img")
            thumb = ""
            if img_elem:
                thumb = img_elem.get("src") or img_elem.get("data-src") or ""

            latest_ch = item.select_one(".epxs")
            manga_type = item.select_one(".type")
            rating = item.select_one(".numscore")

            items.append({
                "title": title,
                "slug": self._normalize_slug(manga_url),
                "url": manga_url,
                "thumbnail": thumb,
                "latest_chapter": latest_ch.text.strip() if latest_ch else "",
                "type": manga_type.text.strip() if manga_type else "",
                "rating": rating.text.strip() if rating else ""
            })

        # Check pagination
        has_next = bool(soup.select_one(".r, a.next"))
        return {
            "page": page,
            "order": order,
            "count": len(items),
            "has_next_page": has_next,
            "results": items
        }

    def get_chapter_images(self, chapter_url_or_slug: str) -> Dict[str, Any]:
        """ดึงรูปภาพทั้งหมดของตอนนั้นๆ"""
        url = chapter_url_or_slug.strip()
        if not url.startswith("http"):
            url = f"{BASE_URL}/{url.strip('/')}/"

        soup = self._fetch_soup(url)

        # 1. Try reading base64 data script
        images = []
        for s in soup.find_all("script", src=True):
            src = s["src"]
            if src.startswith("data:text/javascript;base64,"):
                try:
                    decoded = base64.b64decode(src.split(",", 1)[1]).decode("utf-8", errors="ignore")
                    if "ts_reader.run" in decoded:
                        match = re.search(r"ts_reader\.run\((.*?)\);", decoded, re.DOTALL)
                        if match:
                            data = json.loads(match.group(1))
                            sources = data.get("sources", [])
                            if sources:
                                images = sources[0].get("images", [])
                                break
                except Exception:
                    continue

        # 2. Fallback to inline script or #readerarea img
        if not images:
            for img in soup.select("#readerarea img"):
                src = img.get("src") or img.get("data-src")
                if src:
                    images.append(src)

        title_elem = soup.find("h1", class_="entry-title")
        title = title_elem.text.strip() if title_elem else ""

        return {
            "title": title,
            "url": url,
            "total_images": len(images),
            "images": images
        }

