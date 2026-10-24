// capture-media.mjs — Record 6-second WEBM clips of demo site pages for
// ALICE-Font's README media gallery. Run after starting a local server:
//
//   serve the demo site on port 8765 (python3 -m http.server 8765)
//   cd scripts && node capture-media.mjs
//
// Outputs WEBM files to docs/media/, one per target.
// Post-process each with ffmpeg → optimized GIF (see capture-media README).
//
// Author: Moroya Sakamoto

import { chromium } from 'playwright';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const MEDIA_DIR = path.resolve(__dirname, '..', 'docs', 'media');
const BASE_URL = 'http://localhost:8765';
const RECORD_MS = 6000;
const VIEWPORT = { width: 1200, height: 800 };

// 6 targets: page slug → output filename stem.
const TARGETS = [
  { page: 'font-demo',                 stem: 'font-demo',           scroll: true  },
  { page: 'pattern-sumi',              stem: 'sumi',                scroll: false },
  { page: 'pattern-effekseer-showcase', stem: 'effekseer-showcase', scroll: false },
  { page: 'pattern-liquid-poetry',     stem: 'liquid-poetry',       scroll: false },
  { page: 'pattern-gravity-lens',      stem: 'gravity-lens',        scroll: false },
  { page: 'pattern-cyber-neon',        stem: 'cyber-neon',          scroll: false },
];

async function captureOne(browser, target) {
  const url = `${BASE_URL}/?page=${target.page}`;
  console.log(`[capture] ${target.stem}: ${url}`);

  const context = await browser.newContext({
    viewport: VIEWPORT,
    recordVideo: { dir: MEDIA_DIR, size: VIEWPORT },
  });
  const page = await context.newPage();

  page.on('pageerror', (err) => console.error(`  pageerror: ${err.message}`));
  page.on('console', (msg) => {
    if (msg.type() === 'error') console.error(`  console.error: ${msg.text()}`);
  });

  await page.goto(url, { waitUntil: 'networkidle', timeout: 15000 });
  // Give shader compile + atlas fetch a moment.
  await page.waitForTimeout(1500);

  if (target.scroll) {
    // Font-demo has 144 blocks. Auto-scroll through them during recording.
    const startTime = Date.now();
    while (Date.now() - startTime < RECORD_MS) {
      await page.mouse.wheel(0, 120);
      await page.waitForTimeout(60);
    }
  } else {
    await page.waitForTimeout(RECORD_MS);
  }

  // Grab the video handle before closing the page.
  const videoHandle = page.video();
  await context.close();

  if (videoHandle) {
    const rawPath = await videoHandle.path();
    const dstPath = path.join(MEDIA_DIR, `${target.stem}.webm`);
    await fs.rename(rawPath, dstPath);
    const stat = await fs.stat(dstPath);
    console.log(`  wrote ${dstPath} (${(stat.size / 1024).toFixed(1)} KiB)`);
  } else {
    console.error(`  video handle missing for ${target.stem}`);
  }
}

async function main() {
  await fs.mkdir(MEDIA_DIR, { recursive: true });
  const browser = await chromium.launch({
    headless: true,
    args: [
      '--use-gl=angle',
      '--use-angle=metal',
      '--enable-webgl',
      '--enable-webgl2-compute-context',
    ],
  });
  try {
    for (const target of TARGETS) {
      try {
        await captureOne(browser, target);
      } catch (err) {
        console.error(`[capture] FAILED for ${target.stem}: ${err.message}`);
      }
    }
  } finally {
    await browser.close();
  }
  console.log('[capture] done.');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
