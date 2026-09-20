// The README's screenshots, sized and encoded for the site.
//
// `docs/screenshots/*.png` are 3200×2000 and about 300 KB each — right for
// the README on GitHub, eleven megabytes for a landing page that shows six
// of them. This writes `docs/screenshots/web/<name>.webp` at 1600 px wide,
// which is what the pages display, at a third of the size. Re-run it after
// `npm run screenshots` has reshot the originals:
//
//     node docs/tools/screenshots-web.mjs
//
// Chromium does the encoding (`canvas.toBlob('image/webp')`), because it is
// already a dependency of the repository and cwebp is not.
/* global Buffer, console, Image, document */
import { chromium } from 'playwright';
import { readdirSync, readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve, basename } from 'node:path';

const src = resolve(import.meta.dirname, '../screenshots');
const out = resolve(src, 'web');
mkdirSync(out, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage();
let total = 0;
for (const file of readdirSync(src).filter((f) => f.endsWith('.png'))) {
  // As a data URL: a `file://` image taints the canvas, which then refuses
  // to export.
  const url = 'data:image/png;base64,' + readFileSync(resolve(src, file)).toString('base64');
  const data = await page.evaluate(async (url) => {
    const img = new Image();
    img.src = url;
    await img.decode();
    const width = Math.min(1600, img.naturalWidth);
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = Math.round((img.naturalHeight * width) / img.naturalWidth);
    canvas.getContext('2d').drawImage(img, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise((r) => canvas.toBlob(r, 'image/webp', 0.84));
    return Array.from(new Uint8Array(await blob.arrayBuffer()));
  }, url);
  writeFileSync(resolve(out, basename(file, '.png') + '.webp'), Buffer.from(data));
  total += data.length;
}
await browser.close();
console.log(`${readdirSync(out).length} files, ${(total / 1024 / 1024).toFixed(1)} MB`);
