// verify-browser.mjs — Phase 10-D shader compile + runtime error gate.
//
// Boots demo site pages in headless Chromium (with WebGL2) and reports
// any console.error / pageerror events. Zero errors = shader compiles
// and runs cleanly; any output = regression.
//
// Usage:
//   serve the demo site on port 8765 (python3 -m http.server 8765 &)
//   cd scripts && node verify-browser.mjs

import { chromium } from 'playwright';

const BASE_URL = 'http://localhost:8765';
const TARGETS = [
  { name: 'home',                   url: `${BASE_URL}/?page=home` },
  { name: 'font-demo',              url: `${BASE_URL}/?page=font-demo` },
  { name: 'pattern-sumi',           url: `${BASE_URL}/?page=pattern-sumi` },
  { name: 'pattern-effekseer',      url: `${BASE_URL}/?page=pattern-effekseer-showcase` },
  { name: 'pattern-kino-showcase',  url: `${BASE_URL}/?page=pattern-kino-showcase` },
];

async function verifyOne(browser, target) {
  const context = await browser.newContext({ viewport: { width: 1200, height: 900 } });
  const page = await context.newPage();

  const errors = [];
  const warnings = [];

  page.on('pageerror', (err) => errors.push(`pageerror: ${err.message}`));
  page.on('console', (msg) => {
    const t = msg.type();
    const text = msg.text();
    // Ignore benign 404 for optional atlases (`sans-regular` fallback)
    // and dev-only "asset not preloaded" info messages.
    if (/atlas .* fallback/i.test(text)) return;
    if (t === 'error') errors.push(`console.error: ${text}`);
    else if (t === 'warning') warnings.push(`console.warning: ${text}`);
  });

  try {
    await page.goto(target.url, { waitUntil: 'networkidle', timeout: 20000 });
    // Give the render loop 2 s to hit every conditional branch.
    await page.waitForTimeout(2000);
  } catch (err) {
    errors.push(`goto: ${err.message}`);
  }
  await context.close();
  return { name: target.name, errors, warnings };
}

async function main() {
  const browser = await chromium.launch({
    headless: true,
    args: ['--use-gl=angle', '--use-angle=metal', '--enable-webgl'],
  });
  let failed = 0;
  try {
    for (const target of TARGETS) {
      const r = await verifyOne(browser, target);
      const status = r.errors.length === 0 ? 'PASS' : 'FAIL';
      console.log(`${status}  ${r.name}  (errors=${r.errors.length}, warnings=${r.warnings.length})`);
      r.errors.forEach((e) => console.log(`    ${e}`));
      if (r.warnings.length > 0 && r.errors.length === 0) {
        r.warnings.slice(0, 3).forEach((w) => console.log(`    ${w}`));
      }
      if (r.errors.length > 0) failed++;
    }
  } finally {
    await browser.close();
  }
  console.log(`\nSummary: ${TARGETS.length - failed} pass / ${failed} fail`);
  process.exit(failed > 0 ? 1 : 0);
}

main().catch((err) => { console.error(err); process.exit(1); });
