// capture-media-v2.mjs — Phase 7 rewrite of the README media capture.
//
// Improvements over v1 (capture-media.mjs):
//   - Higher resolution (1200×900) + higher fps (30) + longer duration (8-10s)
//   - Uses the demo site's `?bg=off` URL param to disable the raymarched
//     background so the SDF glyphs / text-effects are the focus.
//   - Injects mouse events (move + click) to actually activate the
//     fluid / physics interactions that pattern-sumi and
//     pattern-effekseer-showcase depend on. Without these events the
//     physics simulations sit idle and the capture ends up showing a
//     static shader rather than the intended physically-driven behaviour.
//   - Output remains WEBM in docs/media/; the sibling script webm-to-mp4.sh
//     converts to MP4 for use in the README <video> tags.
//
// Prerequisites:
//   serve the demo site on port 8765 (python3 -m http.server 8765)
//
// Author: Moroya Sakamoto

import { chromium } from 'playwright';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const MEDIA_DIR = path.resolve(__dirname, '..', 'docs', 'media');
const BASE_URL = 'http://localhost:8765';
const VIEWPORT = { width: 1200, height: 900 };
const RECORD_MS = 8000;

// mouseWander — smoothly moves the cursor along a Lissajous-ish curve
// inside the viewport to trigger continuous fluid density injection.
async function mouseWander(page, durationMs, opts = {}) {
  const { clicksAt = [], amplitude = 0.35 } = opts;
  const startTime = Date.now();
  const centreX = VIEWPORT.width * 0.5;
  const centreY = VIEWPORT.height * 0.55;
  const stepMs = 40;
  let clickIdx = 0;
  await page.mouse.move(centreX, centreY);
  while (Date.now() - startTime < durationMs) {
    const t = (Date.now() - startTime) / 1000;
    const x = centreX + Math.sin(t * 1.3) * VIEWPORT.width * amplitude;
    const y = centreY + Math.cos(t * 0.9) * VIEWPORT.height * amplitude * 0.6;
    await page.mouse.move(x, y);
    while (clickIdx < clicksAt.length && clicksAt[clickIdx].t <= t) {
      const c = clicksAt[clickIdx];
      await page.mouse.click(c.x ?? x, c.y ?? y);
      clickIdx++;
    }
    await page.waitForTimeout(stepMs);
  }
}

async function scrollDown(page, durationMs, pxPerStep = 60, stepMs = 90) {
  const startTime = Date.now();
  while (Date.now() - startTime < durationMs) {
    await page.mouse.wheel(0, pxPerStep);
    await page.waitForTimeout(stepMs);
  }
}

// Six targets — split into "SDF Font itself" (family showcase, static bg
// off) and "SDF Font enables" (physics-driven / animated effects).
const TARGETS = [
  {
    stem: 'font-families',
    // font-demo.json uses light-blue text colors (0.9-1.0 lightness)
    // designed for a dark background — use #0a0e18 (the demo site's default
    // deep-space blue) so glyph samples read clearly.
    url: `${BASE_URL}/?page=font-demo&bg=off&bgcolor=0a0e18`,
    action: async (page) => {
      // Slow, steady scroll so viewers can read individual family samples.
      await scrollDown(page, RECORD_MS, 40, 100);
    },
  },
  {
    stem: 'sumi',
    url: `${BASE_URL}/?page=pattern-sumi`,
    action: async (page) => {
      // Continuous wander → fluid density injection.
      // Click every ~1.5s → ALICE-Physics ink-drop spawn.
      await mouseWander(page, RECORD_MS, {
        clicksAt: [
          { t: 1.0, x: 500, y: 350 },
          { t: 2.5, x: 720, y: 460 },
          { t: 4.2, x: 380, y: 520 },
          { t: 5.8, x: 850, y: 380 },
          { t: 7.0, x: 600, y: 500 },
        ],
      });
    },
  },
  {
    stem: 'effekseer-134-presets',
    url: `${BASE_URL}/?page=pattern-effekseer-showcase&bg=off&bgcolor=0a0e18`,
    action: async (page) => {
      // Click at multiple positions to spawn presets from the tktk01 +
      // Effekseer built-in pools (134 total).
      const positions = [
        { x: 300, y: 400 }, { x: 500, y: 350 }, { x: 700, y: 450 },
        { x: 900, y: 400 }, { x: 400, y: 550 }, { x: 800, y: 550 },
      ];
      for (let i = 0; i < positions.length; i++) {
        await page.mouse.click(positions[i].x, positions[i].y);
        await page.waitForTimeout(RECORD_MS / positions.length);
      }
    },
  },
  {
    stem: 'liquid-morph',
    url: `${BASE_URL}/?page=pattern-liquid-poetry&bg=off&bgcolor=101018`,
    action: async (page) => {
      await mouseWander(page, RECORD_MS, { amplitude: 0.3 });
    },
  },
  {
    stem: 'gravity-lens',
    url: `${BASE_URL}/?page=pattern-gravity-lens&bg=off&bgcolor=05070c`,
    action: async (page) => {
      await mouseWander(page, RECORD_MS, { amplitude: 0.4 });
    },
  },
  {
    stem: 'halo-rings',
    // pattern-alice-thought.json uses the same light-blue palette as
    // font-demo; keep the dark background so text stays legible.
    url: `${BASE_URL}/?page=pattern-alice-thought&bg=off&bgcolor=0a0e18`,
    action: async (page) => {
      // TextEffect::HaloRings + WaterRipple driven by mouse.
      await mouseWander(page, RECORD_MS, {
        clicksAt: [
          { t: 1.5, x: 500, y: 400 },
          { t: 3.5, x: 700, y: 500 },
          { t: 5.5, x: 400, y: 450 },
        ],
      });
    },
  },
  {
    stem: 'kino-showcase',
    // Phase 10-B — 5 Keijiro Kino effects (Datamosh / Contour /
    // AnamorphicStreak / VolumetricFog / Isoline) rendered by the
    // canonical alice_operators.glsl Kino section, consumed by
    // the demo site's text.frag.glsl.
    url: `${BASE_URL}/?page=pattern-kino-showcase`,
    action: async (page) => {
      // Slow scroll through the 5 effect blocks.
      await scrollDown(page, RECORD_MS, 30, 100);
    },
  },
  {
    stem: 'cjk-melt',
    // Phase 10-F — 40-byte MetaFontParams → parametric CJK glyphs
    // (BIZ UDPGothic embedded outline) → TextEffect::MeltDilate /
    // Decompose / Liquid / Sumi in sequence on the same 40 bytes.
    // Demonstrates "same 40 bytes, different fragment-level distortion".
    url: `${BASE_URL}/?page=pattern-cjk-melt`,
    action: async (page) => {
      // Scroll + wander + occasional click to activate Liquid + Sumi
      // effects that depend on cursor position and click events.
      const startTime = Date.now();
      const cx = 600, cy = 500;
      let scrolled = 0;
      await page.mouse.move(cx, cy);
      while (Date.now() - startTime < RECORD_MS) {
        const t = (Date.now() - startTime) / 1000;
        if (Date.now() - startTime > scrolled * 200) {
          await page.mouse.wheel(0, 30);
          scrolled++;
        }
        const x = cx + Math.sin(t * 1.5) * 300;
        const y = cy + Math.cos(t * 1.0) * 150;
        await page.mouse.move(x, y);
        if (Math.floor(t) % 2 === 0 && Math.floor(t * 10) % 20 === 5) {
          await page.mouse.click(x, y);
        }
        await page.waitForTimeout(40);
      }
    },
  },
];

async function captureOne(browser, target) {
  console.log(`[capture] ${target.stem}: ${target.url}`);
  const context = await browser.newContext({
    viewport: VIEWPORT,
    recordVideo: { dir: MEDIA_DIR, size: VIEWPORT },
  });
  const page = await context.newPage();
  page.on('pageerror', (err) => console.error(`  pageerror: ${err.message}`));

  await page.goto(target.url, { waitUntil: 'networkidle', timeout: 20000 });
  await page.waitForTimeout(1500);

  await target.action(page);

  const videoHandle = page.video();
  await context.close();
  if (videoHandle) {
    const rawPath = await videoHandle.path();
    const dstPath = path.join(MEDIA_DIR, `${target.stem}.webm`);
    await fs.rename(rawPath, dstPath);
    const stat = await fs.stat(dstPath);
    console.log(`  wrote ${dstPath} (${(stat.size / 1024).toFixed(1)} KiB)`);
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
