#!/usr/bin/env node
/** Copy the volunteer guide PDF next to the built .app for DMG packaging extras. */
import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const pdf = resolve(root, 'docs/guide/Selah-Volunteer-Guide.pdf');
const destDir = resolve(root, 'src-tauri/target/release/bundle/dmg-extras');
const dest = resolve(destDir, 'Selah-Volunteer-Guide.pdf');

if (!existsSync(pdf)) {
  console.error('Build the guide PDF first: npm run guide:pdf');
  process.exit(1);
}
mkdirSync(destDir, { recursive: true });
copyFileSync(pdf, dest);
console.log(`Copied guide to ${dest}`);
