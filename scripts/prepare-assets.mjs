import { mkdir, readFile, writeFile, access, chmod } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

async function fetchBytes(url) {const r=await fetch(url,{headers:{'User-Agent':'Selah-build/0.1'},signal:AbortSignal.timeout(180000)});if(!r.ok)throw new Error(`Asset download failed: ${r.status} ${url}`);return Buffer.from(await r.arrayBuffer());}
async function exists(path){try{await access(path);return true;}catch{return false;}}
await mkdir('assets/bibles',{recursive:true});await mkdir('assets/runtime',{recursive:true});
for(const id of ['engwebp','spaRV1909']){const name=`${id}_vpl.zip`,path=join('assets/bibles',name);if(!await exists(path)){console.log(`Downloading ${id} from eBible.org…`);await writeFile(path,await fetchBytes(`https://ebible.org/Scriptures/${name}`));}}
const runtime=join('assets/runtime',process.platform==='win32'?'llama-server.exe':'llama-server');
if(!await exists(runtime)){
  const response=await fetch('https://api.github.com/repos/ggml-org/llama.cpp/releases?per_page=10',{headers:{'User-Agent':'Selah-build/0.2',...(process.env.GITHUB_TOKEN?{Authorization:`Bearer ${process.env.GITHUB_TOKEN}`}:{})},signal:AbortSignal.timeout(30000)});if(!response.ok)throw new Error(`Could not load official llama.cpp release manifest (HTTP ${response.status})`);const releases=await response.json();
  const suffix=process.platform==='darwin'?'bin-macos-arm64.tar.gz':process.platform==='win32'?'bin-win-vulkan-x64.zip':null;
  if(!suffix)throw new Error('Runtime packaging supports Apple Silicon macOS and Windows x64');
  // "latest" can point at an unrelated tag, so use the newest release that ships this platform's verified binary.
  const release=releases.find(r=>r.assets.some(a=>a.name.endsWith(suffix)&&a.digest?.startsWith('sha256:')));if(!release)throw new Error(`No verified official runtime asset for ${suffix}`);
  const asset=release.assets.find(a=>a.name.endsWith(suffix));if(!asset||!asset.digest?.startsWith('sha256:'))throw new Error(`No verified official runtime asset for ${suffix}`);
  console.log(`Downloading ${asset.name}…`);const bytes=await fetchBytes(asset.browser_download_url);if(createHash('sha256').update(bytes).digest('hex')!==asset.digest.slice(7))throw new Error('Runtime checksum mismatch');
  const archive=join('assets',asset.name);await writeFile(archive,bytes);
  if(process.platform==='darwin')execFileSync('tar',['-xzf',archive,'--strip-components=1','-C','assets/runtime']);
  else execFileSync('powershell',['-NoProfile','-Command',`Expand-Archive -LiteralPath '${archive.replaceAll("'","''")}' -DestinationPath 'assets/runtime' -Force`]);
  // Release archives may put binaries in a single enclosing folder.
  const { readdir,copyFile }=await import('node:fs/promises');
  async function flatten(dir){for(const e of await readdir(dir,{withFileTypes:true})){const p=join(dir,e.name);if(e.isDirectory())await flatten(p);else if(dir!=='assets/runtime')await copyFile(p,join('assets/runtime',e.name));}}
  await flatten('assets/runtime');
  if(!await exists(runtime))throw new Error('Official archive did not contain llama-server');
  if(process.platform!=='win32')await chmod(runtime,0o755);
  await writeFile('assets/runtime/release.json',JSON.stringify({tag:release.tag_name,asset:asset.name,digest:asset.digest},null,2));
}
console.log('Public-domain Bibles and verified llama.cpp runtime are ready.');
