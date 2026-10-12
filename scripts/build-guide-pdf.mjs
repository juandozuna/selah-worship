#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const html = resolve(root, 'docs/guide/index.html');
const pdf = resolve(root, 'docs/guide/Selah-Volunteer-Guide.pdf');
const chrome =
  process.env.CHROME_PATH ||
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';

if (!existsSync(html)) {
  console.error('Missing docs/guide/index.html');
  process.exit(1);
}
if (!existsSync(chrome)) {
  console.error('Google Chrome not found. Set CHROME_PATH to print the guide PDF.');
  process.exit(1);
}

const result = spawnSync(
  chrome,
  [
    '--headless=new',
    '--disable-gpu',
    '--no-pdf-header-footer',
    `--print-to-pdf=${pdf}`,
    `file://${html}`,
  ],
  { stdio: 'inherit' },
);
process.exit(result.status ?? 1);
