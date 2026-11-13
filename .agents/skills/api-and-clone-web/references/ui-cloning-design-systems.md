# UI/UX Cloning & Design Systems Adaptation

This document details how to reverse-engineer modern web application design systems (e.g. LINE WEBTOON, Netflix, MangaDex) and translate them into a responsive, mobile-first frontend.

---

## 1. Deconstructing the Target Design System

Use `chrome-devtools-mcp` or browser inspection to extract:
1. **Color Tokens**:
   - Primary Accent: `#00dc64` (Signature Webtoon Neon Green)
   - Secondary / Warning / Rating: `#facc15` (Star Gold), `#ef4444` (Heart Red)
   - Background Tiers:
     - Root Darker: `#0a0a0d`
     - Navbar / Floating: `#0f1015` (with `backdrop-blur-md`)
     - Card / Surface: `#181922`
     - Card Hover: `#21222e`
     - Border Subdued: `#2a2b38`
     - Muted Text: `#8e90a0`
2. **Typography**:
   - Primary Font: Google Font `Prompt` (Modern, clean, Thai & Latin support).
   - Hierarchy:
     - Hero Headings: `font-black text-2xl sm:text-4xl tracking-tight`
     - Section Headers: `font-extrabold text-lg sm:text-2xl`
     - Card Titles: `font-bold text-xs sm:text-sm line-clamp-1`
     - Metadata / Badges: `text-[10px] sm:text-[11px] font-semibold`
3. **Card Proportions**:
   - Manga Covers: Always standard `aspect-[3/4]` with `object-cover`.
   - Hover Effects: `hover:-translate-y-1 hover:border-wt-green/60 transition duration-300 shadow-lg`.

---

## 2. Core Layout Components

### 1. Hero Spotlight Carousel
Showcase the #1 trending manga with a blurred backdrop cover, rating badge, synopsis, and quick action buttons:
```html
<section class="relative rounded-2xl sm:rounded-3xl overflow-hidden border border-wt-border bg-wt-card shadow-2xl">
    <div id="heroBanner" class="relative min-h-[260px] sm:min-h-[380px] flex items-end p-4 sm:p-8 lg:p-10">
        <!-- Blurred Cover Background -->
        <div id="heroBg" class="absolute inset-0 bg-cover bg-center opacity-40 scale-105" style="filter: blur(3px);"></div>
        <!-- Gradient Overlay -->
        <div class="absolute inset-0 bg-gradient-to-t from-wt-dark via-wt-dark/60 to-transparent"></div>
        <!-- Hero Details -->
        <div class="relative z-10 max-w-2xl space-y-2">
            <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-wt-green text-black">
                <i class="fa-solid fa-fire"></i> ฮิตที่สุดตอนนี้
            </span>
            <h1 id="heroTitle" class="text-xl sm:text-4xl font-black text-white">...</h1>
            <!-- Action CTAs -->
        </div>
    </div>
</section>
```

### 2. Daily Comic Schedule Tabs (Horizontal Momentum Swiping)
On mobile devices, daily tabs must allow natural horizontal swiping without scrollbars:
```html
<div class="flex items-center gap-1 overflow-x-auto pb-1 sm:pb-0 hide-scrollbar -mx-3 px-3 sm:mx-0 sm:px-0" id="weekdayTabs">
    <button onclick="filterByDay('ALL')" class="day-tab flex-shrink-0 px-3.5 py-1.5 rounded-full text-xs font-semibold bg-wt-green text-black">ทั้งหมด</button>
    <button onclick="filterByDay('จันทร์')" class="day-tab flex-shrink-0 px-3.5 py-1.5 rounded-full text-xs font-semibold text-wt-muted hover:text-white">จันทร์</button>
    <!-- ... -->
</div>
```

### 3. Top 5 Ranking Section
Cards with stylized rank badges (1, 2, 3 highlighted in neon green, 4 and 5 in dark gray):
```html
<span class="absolute top-2 left-2 w-7 h-7 flex items-center justify-center rounded-lg font-black text-xs ${idx < 3 ? 'bg-wt-green text-black shadow-lg shadow-wt-green/40' : 'bg-wt-dark/90 text-white border border-wt-border'}">
    ${idx + 1}
</span>
```

### 4. Responsive Manga Grid (Mobile 2-Column, Desktop 6-Column)
Ensure touch targets are well-spaced and cards never stretch unnaturally:
```html
<div id="mangaGrid" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2.5 sm:gap-4">
    <!-- Populated dynamically -->
</div>
```

---

## 3. Read Status Tracking ("อ่านแล้ว ✓")

Readers expect to immediately see which chapters they have already consumed.

### Implementation:
```javascript
// Storage Helper
function getReadChapters(slug) {
    try {
        const store = JSON.parse(localStorage.getItem('webtoon_read_chapters') || '{}');
        return store[slug] || [];
    } catch (e) {
        return [];
    }
}

function markChapterAsRead(slug, chapterUrl) {
    try {
        const store = JSON.parse(localStorage.getItem('webtoon_read_chapters') || '{}');
        if (!store[slug]) store[slug] = [];
        if (!store[slug].includes(chapterUrl)) {
            store[slug].push(chapterUrl);
            localStorage.setItem('webtoon_read_chapters', JSON.stringify(store));
        }
    } catch (e) {}
}

// Rendering Chapter Badges
chapters.forEach(ch => {
    const isRead = getReadChapters(currentManga.slug).includes(ch.url);
    const badgeHtml = isRead ? `
        <span class="text-[9px] font-bold px-2 py-0.5 rounded-full bg-wt-green/20 text-wt-green border border-wt-green/30 flex items-center gap-1">
            <i class="fa-solid fa-check text-[8px]"></i> อ่านแล้ว
        </span>
    ` : '';
    // Append styled row
});
```

---

## 4. Vector Icon Standardization (Font Awesome 6)

Never use raw emojis in modern production apps; emojis render inconsistently across Android, iOS, Windows, and Linux.

Replace all emojis with Font Awesome 6 icons:
```html
<!-- In Head -->
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">

<!-- Examples -->
<i class="fa-solid fa-house text-xs text-wt-green"></i>
<i class="fa-solid fa-magnifying-glass"></i>
<i class="fa-solid fa-fire text-amber-400"></i>
<i class="fa-solid fa-star text-amber-400"></i>
<i class="fa-solid fa-bolt text-wt-green"></i>
<i class="fa-solid fa-clock-rotate-left"></i>
<i class="fa-solid fa-chevron-left"></i>
<i class="fa-solid fa-chevron-right"></i>
```
