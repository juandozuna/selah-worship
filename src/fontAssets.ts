import { appDataDir, join } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { FontAsset } from './types';

const registered = new Map<string, Promise<boolean>>();
/** Native renderers register their own copy; CSS registration is best effort under the webview CSP. */
export function registerFontAsset(asset: FontAsset): Promise<boolean> {
  const existing = registered.get(asset.id);
  if (existing) return existing;
  const attempt = (async () => {
    if (typeof FontFace === 'undefined' || !document.fonts) return false;
    try {
      const path = await join(await appDataDir(), 'media', asset.relativePath);
      const face = new FontFace(asset.family, `local(${JSON.stringify(asset.family)}), url(${JSON.stringify(convertFileSrc(path))})`);
      await face.load();
      document.fonts.add(face);
      window.dispatchEvent(new Event('selah:fonts-loaded'));
      return true;
    } catch { return false; }
  })();
  registered.set(asset.id, attempt);
  return attempt;
}
export function registerProjectFonts(assets: FontAsset[]): void {
  for (const asset of assets) void registerFontAsset(asset);
}
