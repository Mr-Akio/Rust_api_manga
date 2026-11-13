from typing import List, Optional
from pydantic import BaseModel, Field


class ChapterItem(BaseModel):
    chapter_number: str = Field(..., description="หมายเลขตอน เช่น '252' หรือ '1'")
    title: str = Field(..., description="ชื่อตอน เช่น 'ตอนที่ 252'")
    url: str = Field(..., description="ลิงก์หน้าอ่านของตอนนั้น")
    date: str = Field(..., description="วันที่เผยแพร่/อัปเดต")


class MangaDetailResponse(BaseModel):
    title: str = Field(..., description="ชื่อเรื่องมังงะ")
    alternative_title: Optional[str] = Field("", description="ชื่อเรื่องภาษาอื่นๆ")
    slug: str = Field(..., description="สลัก (Slug) ของมังงะ")
    url: str = Field(..., description="URL ต้นทางบนเว็บ 1668manga.com")
    thumbnail: Optional[str] = Field("", description="URL รูปภาพหน้าปก")
    rating: Optional[str] = Field("", description="คะแนนรีวิว")
    synopsis: Optional[str] = Field("", description="เรื่องย่อ")
    genres: List[str] = Field(default_factory=list, description="หมวดหมู่")
    status: Optional[str] = Field("", description="สถานะ เช่น Ongoing, Completed")
    type: Optional[str] = Field("", description="ประเภท เช่น Manga, Manhwa, Manhua")
    author: Optional[str] = Field("", description="ผู้แต่ง/ผู้เขียน")
    artist: Optional[str] = Field("", description="ผู้วาด")
    released: Optional[str] = Field("", description="ปีที่เริ่มเผยแพร่")
    serialization: Optional[str] = Field("", description="สำนักพิมพ์/นิตยสาร")
    updated_at: Optional[str] = Field("", description="วันที่อัปเดตล่าสุด")
    total_chapters: int = Field(..., description="จำนวนตอนทั้งหมด")
    chapters: List[ChapterItem] = Field(default_factory=list, description="รายการตอนทั้งหมด เรียงจากล่าสุดไปตอนแรก")


class MangaSearchResult(BaseModel):
    title: str
    slug: str
    url: str
    thumbnail: Optional[str] = ""
    latest_chapter: Optional[str] = ""
    type: Optional[str] = ""
    rating: Optional[str] = ""


class MangaListResponse(BaseModel):
    page: int
    order: str
    count: int
    has_next_page: bool
    results: List[MangaSearchResult]


class ChapterImagesResponse(BaseModel):
    title: str
    url: str
    total_images: int
    images: List[str]

