import os
import time
from typing import Optional, Dict, Any
import requests
from fastapi import FastAPI, HTTPException, Query, Path
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse, StreamingResponse, JSONResponse
from fastapi.staticfiles import StaticFiles

from scraper import MangaScraper
from models import (
    MangaDetailResponse,
    MangaSearchResult,
    MangaListResponse,
    ChapterImagesResponse
)

app = FastAPI(
    title="1668Manga Reader & API Service",
    description="API & Web Reader สำหรับดึงข้อมูลชื่อเรื่อง และทุกตอนของมังงะจาก https://1668manga.com/",
    version="1.1.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# Enable CORS for cross-origin requests
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Static files
STATIC_DIR = os.path.join(os.path.dirname(__file__), "static")
if os.path.exists(STATIC_DIR):
    app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

scraper = MangaScraper()

# Simple TTL In-Memory Cache (5 minutes)
CACHE_TTL = 300  # seconds
cache: Dict[str, Dict[str, Any]] = {}

def get_from_cache(key: str) -> Optional[Any]:
    entry = cache.get(key)
    if entry and (time.time() - entry["timestamp"] < CACHE_TTL):
        return entry["data"]
    if entry:
        del cache[key]
    return None

def set_cache(key: str, data: Any):
    if len(cache) > 500:
        oldest_key = min(cache.keys(), key=lambda k: cache[k]["timestamp"])
        del cache[oldest_key]
    cache[key] = {"data": data, "timestamp": time.time()}


@app.get("/", response_class=FileResponse, summary="หน้าเว็บอ่านมังงะ (Manga Reader Web App)")
@app.get("/reader", response_class=FileResponse, summary="หน้าเว็บอ่านมังงะ (Manga Reader Web App)")
def serve_reader():
    index_path = os.path.join(STATIC_DIR, "index.html")
    if os.path.exists(index_path):
        return FileResponse(index_path, media_type="text/html")
    raise HTTPException(status_code=404, detail="static/index.html not found")


@app.get(
    "/api/manga/{slug}",
    response_model=MangaDetailResponse,
    summary="ดึงชื่อเรื่อง และรายการตอนทั้งหมดตาม Slug",
    description="ส่ง slug ของมังงะ (เช่น `dandadan` หรือ `one-piece`) เพื่อรับชื่อเรื่อง รูปปก เรื่องย่อ และรายการตอนทั้งหมด"
)
def get_manga_by_slug(
    slug: str = Path(..., description="Slug ของมังงะ เช่น 'dandadan' หรือ 'mechanic-slayer'")
):
    cache_key = f"manga:{slug.lower()}"
    cached = get_from_cache(cache_key)
    if cached:
        return cached

    try:
        data = scraper.get_manga_detail(slug)
        set_cache(cache_key, data)
        return data
    except ValueError as ve:
        raise HTTPException(status_code=404, detail=str(ve))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get(
    "/api/manga",
    response_model=MangaDetailResponse,
    summary="ดึงชื่อเรื่อง และรายการตอนทั้งหมดด้วย URL หรือ Slug",
    description="ส่ง URL เต็ม เช่น `https://1668manga.com/manga/dandadan/` หรือ slug ผ่าน query parameter"
)
def get_manga_by_query(
    url: Optional[str] = Query(None, description="URL เต็มของมังงะ"),
    slug: Optional[str] = Query(None, description="Slug ของมังงะ")
):
    target = url or slug
    if not target:
        raise HTTPException(status_code=400, detail="กรุณาระบุพารามิเตอร์ url หรือ slug")

    cache_key = f"manga:{target.lower()}"
    cached = get_from_cache(cache_key)
    if cached:
        return cached

    try:
        data = scraper.get_manga_detail(target)
        set_cache(cache_key, data)
        return data
    except ValueError as ve:
        raise HTTPException(status_code=404, detail=str(ve))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get(
    "/api/search",
    response_model=list[MangaSearchResult],
    summary="ค้นหามังงะตามชื่อ",
    description="ค้นหาการ์ตูน/มังงะจากเว็บ 1668manga.com ด้วยคำค้นหา"
)
def search_manga(
    q: str = Query(..., description="คำค้นหา เช่น solo, one piece, dandadan")
):
    cache_key = f"search:{q.lower()}"
    cached = get_from_cache(cache_key)
    if cached:
        return cached

    try:
        results = scraper.search(q)
        set_cache(cache_key, results)
        return results
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get(
    "/api/latest",
    response_model=MangaListResponse,
    summary="ดึงรายการมังงะอัปเดตล่าสุด",
    description="ดึงรายการมังงะหน้าแคตตาล็อก เรียงตามการอัปเดตตอนใหม่"
)
def list_latest(
    page: int = Query(1, ge=1, description="เลขหน้าเริ่มต้นที่ 1"),
    order: str = Query("update", description="การเรียงลำดับ เช่น update, popular, title")
):
    cache_key = f"list:{page}:{order}"
    cached = get_from_cache(cache_key)
    if cached:
        return cached

    try:
        data = scraper.list_mangas(page=page, order=order)
        set_cache(cache_key, data)
        return data
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get(
    "/api/chapter",
    response_model=ChapterImagesResponse,
    summary="ดึงรูปรวมในแต่ละตอน",
    description="ดึง URL รูปภาพทั้งหมดของตอนนั้นเพื่อนำไปเปิดอ่าน"
)
def get_chapter_pages(
    url: str = Query(..., description="URL หรือ path ของตอนนั้น เช่น https://1668manga.com/dandadan-ตอนที่-252/")
):
    cache_key = f"chapter:{url.lower()}"
    cached = get_from_cache(cache_key)
    if cached:
        return cached

    try:
        data = scraper.get_chapter_images(url)
        set_cache(cache_key, data)
        return data
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get(
    "/api/proxy-image",
    summary="Proxy รูปภาพเพื่อป้องกันการติดบล็อก Referer / CORS",
    description="ดาวน์โหลดรูปภาพจาก CDN โดยไม่ส่ง Referer และส่งคืนกลับไปยังเบราว์เซอร์"
)
def proxy_image(
    url: str = Query(..., description="URL ของรูปภาพ")
):
    try:
        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
        }
        resp = requests.get(url, headers=headers, stream=True, timeout=12)
        resp.raise_for_status()
        content_type = resp.headers.get("content-type", "image/jpeg")
        return StreamingResponse(resp.iter_content(chunk_size=16384), media_type=content_type)
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Proxy error: {str(e)}")


if __name__ == "__main__":
    import uvicorn
    import os
    port = int(os.environ.get("PORT", 8000))
    print(f"🚀 เซิร์ฟเวอร์เริ่มทำงานที่ http://0.0.0.0:{port}")
    print(f"📖 หน้าเว็บอ่านการ์ตูน: http://localhost:{port}/")
    print(f"📑 Swagger UI Docs: http://localhost:{port}/docs")
    uvicorn.run("main:app", host="0.0.0.0", port=port, reload=True)
