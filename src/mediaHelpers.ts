import type { AudioDevice, Layout, MediaAsset, MediaKind, MediaLayer, MediaRole, Project } from './types';
import { desktop, invoke } from './bridge';

export function fullBleedLayer(
  role: MediaRole,
  kind: MediaKind,
  assetId: string,
  layout: Layout,
  name?: string,
): MediaLayer {
  return {
    id: crypto.randomUUID(),
    name: name ?? (role === 'background' ? 'Background' : 'Foreground'),
    role,
    kind,
    assetId,
    x: 0,
    y: 0,
    width: layout.width,
    height: layout.height,
    opacity: role === 'foreground' ? 0.85 : 1,
    muted: true,
    audioDeviceId: null,
    fit: 'cover',
  };
}

export async function importMedia(project: Project): Promise<{ project: Project; asset: MediaAsset }> {
  if (desktop()) {
    const asset = await invoke<MediaAsset>('import_media_file');
    return { project: { ...project, media: [...(project.media ?? []), asset] }, asset };
  }
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*,video/*';
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) {
        reject(new Error('No file selected'));
        return;
      }
      if (file.size > 80_000_000) {
        reject(new Error('Choose a media file under 80 MB.'));
        return;
      }
      const kind: MediaKind = file.type.startsWith('video/') ? 'video' : 'image';
      const reader = new FileReader();
      reader.onload = () => {
        const asset: MediaAsset = {
          id: crypto.randomUUID(),
          name: file.name,
          path: String(reader.result),
          kind,
          mime: file.type || (kind === 'video' ? 'video/mp4' : 'image/png'),
        };
        resolve({ project: { ...project, media: [...(project.media ?? []), asset] }, asset });
      };
      reader.onerror = () => reject(new Error('Could not read media file'));
      reader.readAsDataURL(file);
    };
    input.click();
  });
}

export async function listAudioOutputs(): Promise<AudioDevice[]> {
  if (desktop()) {
    try {
      return await invoke<AudioDevice[]>('list_audio_devices');
    } catch {
      /* fall through */
    }
  }
  try {
    const list = await navigator.mediaDevices.enumerateDevices();
    return list
      .filter((d) => d.kind === 'audiooutput')
      .map((d) => ({ id: d.deviceId, label: d.label || 'Audio output' }));
  } catch {
    return [{ id: 'default', label: 'System default' }];
  }
}

export function upsertMediaLayer(layout: Layout, layer: MediaLayer): Layout {
  const layers = [...(layout.mediaLayers ?? [])];
  const i = layers.findIndex((l) => l.id === layer.id);
  if (i >= 0) layers[i] = layer;
  else layers.push(layer);
  return { ...layout, mediaLayers: layers };
}

export function removeMediaLayer(layout: Layout, id: string): Layout {
  return { ...layout, mediaLayers: (layout.mediaLayers ?? []).filter((l) => l.id !== id) };
}
