#!/usr/bin/env node
/**
 * Capture real Selah UI screenshots for the landing page, README, and volunteer PDF.
 * Uses the browser preparation surface (same React UI as the desktop app).
 */
import { createServer } from 'node:http';
import { readFile, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { createReadStream, existsSync, statSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { chromium } from 'playwright';

const root = resolve(import.meta.dirname, '..');
const outGuide = join(root, 'docs/guide/images');
const outSite = join(root, 'site/assets/screenshots');
const outReadme = join(root, 'docs/screenshots');

const mime = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.json': 'application/json',
  '.woff2': 'font/woff2',
};

async function serveDist() {
  const dist = join(root, 'dist');
  const server = createServer((req, res) => {
    const url = decodeURIComponent((req.url || '/').split('?')[0]);
    let path = join(dist, url === '/' ? 'index.html' : url.replace(/^\//, ''));
    if (!path.startsWith(dist) || !existsSync(path) || statSync(path).isDirectory()) {
      path = join(dist, 'index.html');
    }
    res.writeHead(200, { 'Content-Type': mime[extname(path)] || 'application/octet-stream' });
    createReadStream(path).pipe(res);
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const { port } = server.address();
  return { server, base: `http://127.0.0.1:${port}` };
}

async function dismissWelcome(page) {
  const start = page.getByRole('button', { name: /start|comenzar|begin/i });
  if (await start.count()) {
    await start.first().click();
    await page.waitForTimeout(400);
  }
  // Click backdrop / close if still open
  const close = page.locator('dialog[open] button, .modal button').filter({ hasText: /close|×|start/i });
  if (await close.count()) {
    try {
      await close.first().click({ timeout: 1000 });
    } catch {
      /* already closed */
    }
  }
}

async function shot(page, name, fullPage = false) {
  await page.waitForTimeout(500);
  const file = `${name}.png`;
  const path = join(outGuide, file);
  await page.screenshot({ path, fullPage, type: 'png' });
  await copyFile(path, join(outSite, file));
  await copyFile(path, join(outReadme, file));
  console.log('captured', file);
  return file;
}

async function main() {
  await mkdir(outGuide, { recursive: true });
  await mkdir(outSite, { recursive: true });
  await mkdir(outReadme, { recursive: true });

  // Seed localStorage so welcome is skipped when possible
  const { server, base } = await serveDist();
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
  });
  await context.addInitScript(() => {
    try {
      localStorage.setItem('selah.welcome', '1');
    } catch {
      /* ignore */
    }
  });
  const page = await context.newPage();
  await page.goto(base, { waitUntil: 'networkidle' });
  await dismissWelcome(page);
  await page.waitForSelector('.app-shell, .live-workspace, header', { timeout: 15000 });

  // Present view with a live block if possible
  const liveNav = page.getByRole('button', { name: /^live|presentar|present/i }).first();
  if (await liveNav.count()) await liveNav.click();
  await page.waitForTimeout(300);
  const block = page.locator('.lyric-block').first();
  if (await block.count()) {
    await block.click();
    await page.waitForTimeout(600);
  }
  await shot(page, '01-present');

  // Layout editor
  const layoutNav = page.getByRole('button', { name: /layout|diseño/i }).first();
  await layoutNav.click();
  await page.waitForTimeout(700);
  await shot(page, '02-layout');

  // Media section focus — scroll sidebar media controls into view if present
  const mediaBtn = page.getByRole('button', { name: /background media|foreground media|media/i }).first();
  if (await mediaBtn.count()) {
    await mediaBtn.scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
  }
  await shot(page, '03-media');

  // Settings / output (ProPresenter path)
  const settingsNav = page.getByRole('button', { name: /settings|ajustes/i }).first();
  await settingsNav.click();
  await page.waitForTimeout(700);
  await shot(page, '04-settings');

  // Library
  const libNav = page.getByRole('button', { name: /library|biblioteca/i }).first();
  await libNav.click();
  await page.waitForTimeout(700);
  await shot(page, '05-library');

  // Clear live and capture idle present again for install-style overview
  await liveNav.click();
  await page.waitForTimeout(400);
  const clear = page.getByRole('button', { name: /clear|limpiar/i }).first();
  if (await clear.count()) await clear.click();
  await page.waitForTimeout(400);
  await shot(page, '06-overview');

  await browser.close();
  server.close();

  const manifest = {
    capturedAt: new Date().toISOString(),
    files: [
      '01-present.png',
      '02-layout.png',
      '03-media.png',
      '04-settings.png',
      '05-library.png',
      '06-overview.png',
    ],
  };
  await writeFile(join(outGuide, 'manifest.json'), JSON.stringify(manifest, null, 2));
  console.log('Screenshots ready in docs/guide/images, site/assets/screenshots, docs/screenshots');
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
