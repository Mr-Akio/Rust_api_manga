# Webtoon Reader Engine & Subpixel Seam Stitching

This document outlines the engineering principles and CSS implementation required to render spliced continuous webtoon images seamlessly without hairline fracture lines, stuttering, or layout shifts.

---

## 1. The Anatomy of Webtoon Artwork Slicing

Webtoon artists and platforms export episodes not as one massive 50,000px image (which would crash GPU texture memory), but sliced into 40–80 individual JPEG/WebP slices of 800px × 1,200px.

Because the art is continuous across slices, speech bubbles and background art frequently cross slice boundaries.

---

## 2. Why the Seam / Gap Appears (The Subpixel Problem)

When images are displayed in HTML with:
```css
.manga-page-img {
    display: block;
    width: 100%;
    max-width: 800px;
}
```

On any screen where the physical pixel width does not divide cleanly by the aspect ratio, or where the device pixel ratio (DPR) is non-integer (e.g. 1.25x, 1.5x, 1.75x, 2.25x on Windows laptops and Android phones):
1. **Fractional Heights**: A 1,200px tall image scaled to 390px viewport becomes `585.475px`.
2. **Rounding Inconsistency**:
   - Image 1 ends at: `585.475px` (Browser rounds down to pixel 585).
   - Image 2 starts at: `585.475px` (Browser rounds up to pixel 586).
3. **The Result**: A 0.5px to 1px row of unpainted physical pixels between the two images.
   - The container's dark background color (`#000` or `#12131a`) shines through as an annoying horizontal gray or black line, cutting right through speech bubbles and characters.

---

## 3. The 100% Zero-Seam Solution

To completely destroy the subpixel seam across all screen densities, zoom levels, and devices, apply the following 4-tier CSS rule:

```css
/* Tier 1: Destroy all typography and whitespace gaps in parent */
#readerImagesContainer {
    font-size: 0 !important;
    line-height: 0 !important;
    letter-spacing: 0 !important;
    word-spacing: 0 !important;
    padding-top: 0 !important;
    padding-bottom: 0 !important;
}

/* Tier 2: Force GPU block alignment on all image slices */
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

/* Tier 3: 1.5px Negative Margin Overlap */
.manga-page-img + .manga-page-img,
.manga-page-img:not(:first-child) {
    margin-top: -1.5px !important;
}
```

### Why `-1.5px` Overlap Works:
- On a 1x standard display: 1.5px overlap consumes 1 to 2 physical pixels. On a 1,200px tall image, this is `0.1%` distortion—completely imperceptible to the human eye.
- On a 2x or 3x Retina display: 1.5 CSS px = 3 to 4.5 physical pixels.
- Because slice N+1 physically covers the bottom edge of slice N by 1.5 pixels, **there is physically no gap for the background to peek through**. Speech bubbles and linework connect seamlessly.

---

## 4. Reader Performance: Memory & Lazy Loading

Rendering 60–80 full-resolution images simultaneously can cause memory pressure on low-tier mobile devices.

### Recommended Loading Strategy:
```javascript
images.forEach((imgUrl, i) => {
    const img = document.createElement('img');
    img.className = 'manga-page-img';
    img.alt = `หน้า ${i + 1}`;
    
    // Eager load the first 3 images for instant viewing; lazy load the rest
    img.loading = (i < 3) ? 'eager' : 'lazy';
    img.decoding = 'async'; // Non-blocking image decode
    
    img.src = imgUrl;
    img.onerror = () => {
        img.onerror = null;
        img.src = `/api/proxy-image?url=${encodeURIComponent(imgUrl)}`;
    };

    container.appendChild(img);
});
```

---

## 5. Continuous Scroll Reader UX Best Practices

### 1. Auto-Scroll to Top on Chapter Change
When a user finishes reading chapter X, their scroll position is at the very bottom (e.g. `scrollY: 24,000px`). When they tap "ตอนต่อไป ❯":
- Immediately invoke `window.scrollTo({ top: 0, behavior: 'instant' });`
- When the new chapter's images finish rendering, invoke `window.scrollTo({ top: 0, behavior: 'instant' })` again inside `requestAnimationFrame`:
```javascript
async function openChapter(chapter, index) {
    window.scrollTo({ top: 0, behavior: 'instant' });
    // ... fetch & render ...
    renderReaderImages(data.images);
    window.scrollTo({ top: 0, behavior: 'instant' });
    requestAnimationFrame(() => {
        window.scrollTo({ top: 0, behavior: 'instant' });
    });
}
```

### 2. Auto-Hiding Toolbars (Webtoons Standard)
- Scrolling down: Slide top header `-translateY(100%)` and bottom toolbar `translateY(100%)` to immerse the reader.
- Scrolling up: Slide both toolbars back in (`translateY(0)`).
- Tap on reader canvas: Toggle toolbar visibility on/off.

### 3. End-of-Chapter Card
At the bottom of the comic strip, place a prominent End Card:
- Badge: `🎉 จบตอนนี้แล้ว`
- Primary CTA: **`ตอนต่อไป (ตอนที่ X) ❯`** (Large green button with `active:scale-95`).
- Secondary CTA: **`❮ ตอนก่อนหน้า`**
- Utility links: **`♥ ถูกใจตอนนี้`**, **`☰ รายการตอนทั้งหมด`**, **`⬆ กลับด้านบน`**.
