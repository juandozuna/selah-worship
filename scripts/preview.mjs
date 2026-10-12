import { build } from 'esbuild';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
const result=await build({entryPoints:['src/main.tsx'],bundle:true,write:false,format:'iife',jsx:'automatic',loader:{'.css':'empty'},define:{'process.env.NODE_ENV':'"production"'},minify:true});
const css=await readFile('src/styles.css','utf8');
await mkdir('artifacts',{recursive:true});
await writeFile('artifacts/preview.html',`<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Selah · Browser preview</title><style>${css}</style></head><body><div id="root"></div><script>${result.outputFiles[0].text.replace(/<\/script/gi,'<\\/script')}</script></body></html>`);
console.log('Open artifacts/preview.html in a browser. Desktop integrations require the Tauri app.');
