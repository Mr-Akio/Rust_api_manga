# 1668Manga API & Web Reader (Rust Edition) 🦀⚡

ระบบ API และเว็บอ่านการ์ตูน/มังงะจาก [https://1668manga.com/](https://1668manga.com/) พัฒนาด้วยภาษา **Rust (Axum + Tokio + Scraper)** 
- 🚀 **เร็ว แรง กินแรมน้อยมาก** (~15-20 MB เท่านั้น)
- 📦 **คอมไพล์เป็นไฟล์ `.exe` ตัวเดียว** ขนาดเพียง 5.6 MB รันได้ทันทีโดยไม่ต้องลง Python หรือโปรแกรมใดๆ เพิ่มเติม
- 🛡️ **Bypass Cloudflare Bot Detection** ได้อย่างสมบูรณ์แบบ
- 📖 **มาพร้อมเว็บอ่านการ์ตูน (Web Reader SPA)** สไตล์โมเดิร์น Dark Mode ในตัว

---

## 📂 โครงสร้างโปรเจกต์ Rust

```text
d:\Api_test\
├── Cargo.toml                 # กำหนด Dependencies ของ Rust
├── src/
│   ├── main.rs               # Axum Server, Routes, In-memory Cache & Handlers
│   ├── scraper.rs            # Scraper ดึงข้อมูลชื่อเรื่อง ตอน และรูปภาพ
│   └── models.rs             # Structs ข้อมูล (Serde Serialize/Deserialize)
├── static/
│   └── index.html            # หน้าเว็บอ่านมังงะ (Manga Reader SPA)
├── target/release/
│   └── manga_api.exe         # ไฟล์ Release Binary พร้อมใช้งาน (5.6 MB)
└── README.md
```

---

## 🏃 วิธีการรันใช้งาน

### วิธีที่ 1: รันผ่านไฟล์ `.exe` ทันที (ไม่ต้องใช้ Cargo)
สามารถเปิด Terminal แล้วรัน:
```powershell
d:\Api_test\target\release\manga_api.exe
```
หรือดับเบิลคลิกไฟล์ `target\release\manga_api.exe` ได้เลย!

### วิธีที่ 2: รันผ่านคำสั่ง Cargo (Development)
```bash
cargo run --release
```

---

## 🌐 ลิงก์เข้าใช้งาน
- 📖 **เว็บอ่านมังงะ (Web Reader)**: [http://localhost:8000/](http://localhost:8000/) หรือ [http://localhost:8000/reader](http://localhost:8000/reader)
- 📡 **API Service**: [http://localhost:8000/api/...](http://localhost:8000/api/latest)

---

## 📡 รายละเอียด API Endpoints

### 1. ดึงชื่อเรื่อง และทุกตอนของมังงะ
`GET /api/manga/{slug}` หรือ `GET /api/manga?url={manga_url}`

**ตัวอย่าง:**
```bash
curl http://localhost:8000/api/manga/dandadan
```
**ผลลัพธ์ (JSON):**
```json
{
  "title": "Dandadan",
  "alternative_title": "Dan Da Dan...",
  "slug": "dandadan",
  "url": "https://1668manga.com/manga/dandadan/",
  "thumbnail": "https://cdn.1668manga.com/2025/03/Dandadan.avif",
  "rating": "8.2",
  "synopsis": "อายาเซะ โมโมะ เด็กสาวผู้เชื่อในเรื่องของผีสาง...",
  "genres": ["Action", "Comedy", "Drama", "Shounen"],
  "status": "Ongoing",
  "type": "Manga",
  "total_chapters": 253,
  "chapters": [
    {
      "chapter_number": "252",
      "title": "ตอนที่ 252",
      "url": "https://1668manga.com/dandadan-%e0%b8%95%e0%b8%ad%e0%b8%99%e0%b8%97%e0%b8%b5%e0%b9%88-252/",
      "date": "November 10, 2026"
    }
  ]
}
```

### 2. ค้นหามังงะตามชื่อเรื่อง
`GET /api/search?q={keyword}`
```bash
curl "http://localhost:8000/api/search?q=one%20piece"
```

### 3. รายการมังงะอัปเดตล่าสุด
`GET /api/latest?page=1&order=update`
```bash
curl "http://localhost:8000/api/latest"
```

### 4. ดึงรูปภาพทั้งหมดของตอนนั้น (สำหรับเปิดอ่าน)
`GET /api/chapter?url={chapter_url}`
```bash
curl "http://localhost:8000/api/chapter?url=https://1668manga.com/dandadan-%e0%b8%95%e0%b8%ad%e0%b8%99%e0%b8%97%e0%b8%b5%e0%b9%88-252/"
```

### 5. Proxy รูปภาพ (ป้องกัน Hotlink/Referer Block)
`GET /api/proxy-image?url={image_url}`
