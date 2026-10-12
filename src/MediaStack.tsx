import React, { useEffect, useRef, useState } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { AudioDevice, Layout, MediaAsset, MediaLayer } from './types';
import { assetUrl } from './model';
import { desktop, invoke } from './bridge';

function resolveSrc(asset: MediaAsset | undefined, urls: Record<string, string>): string | null {
  if (!asset) return null;
  if (urls[asset.id]) return urls[asset.id];
  if (desktop() && asset.path.startsWith('/')) {
    try {
      return convertFileSrc(asset.path);
    } catch {
      /* fall through */
    }
  }
  return assetUrl(asset);
}

function MediaElement({
  layout,
  layer,
  src,
  pump,
  devices,
}: {
  layout: Layout;
  layer: MediaLayer;
  src: string;
  pump: boolean;
  devices: AudioDevice[];
}) {
  const ref = useRef<HTMLVideoElement | HTMLImageElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el || layer.kind !== 'video') return;
    const video = el as HTMLVideoElement;
    video.muted = layer.muted;
    video.loop = true;
    video.playsInline = true;
    video.play().catch(() => {});
    const sink = layer.audioDeviceId || devices[0]?.id;
    const withSink = video as HTMLVideoElement & { setSinkId?: (id: string) => Promise<void> };
    if (sink && typeof withSink.setSinkId === 'function') {
      withSink.setSinkId(sink).catch(() => {});
    }
  }, [layer.muted, layer.audioDeviceId, layer.kind, src, devices]);
  useEffect(() => {
    if (!pump || layer.kind !== 'video' || !desktop()) return;
    const video = ref.current as HTMLVideoElement | null;
    if (!video) return;
    const canvas = document.createElement('canvas');
    let alive = true;
    let timer = 0;
    const tick = () => {
      if (!alive) return;
      if (video.readyState >= 2 && video.videoWidth > 0) {
        const w = Math.min(1280, video.videoWidth);
        const h = Math.max(1, Math.round((w * video.videoHeight) / video.videoWidth));
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext('2d');
        if (ctx) {
          ctx.drawImage(video, 0, 0, w, h);
          const data = ctx.getImageData(0, 0, w, h).data;
          let binary = '';
          const chunk = 0x8000;
          for (let i = 0; i < data.length; i += chunk) {
            binary += String.fromCharCode(...data.subarray(i, i + chunk));
          }
          invoke('push_media_frame', {
            layerId: layer.id,
            width: w,
            height: h,
            rgbaBase64: btoa(binary),
          }).catch(() => {});
        }
      }
      timer = window.setTimeout(tick, 100);
    };
    tick();
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [pump, layer.id, layer.kind, src]);
  const style: React.CSSProperties = {
    position: 'absolute',
    left: `${(layer.x / layout.width) * 100}%`,
    top: `${(layer.y / layout.height) * 100}%`,
    width: `${(layer.width / layout.width) * 100}%`,
    height: `${(layer.height / layout.height) * 100}%`,
    opacity: layer.opacity,
    objectFit: (layer.fit ?? 'cover') as React.CSSProperties['objectFit'],
    pointerEvents: 'none',
  };
  if (layer.kind === 'video') {
    return (
      <video
        ref={ref as React.RefObject<HTMLVideoElement>}
        className={`media-layer media-${layer.role}`}
        src={src}
        style={style}
        muted={layer.muted}
        loop
        playsInline
        autoPlay
      />
    );
  }
  return (
    <img
      ref={ref as React.RefObject<HTMLImageElement>}
      className={`media-layer media-${layer.role}`}
      src={src}
      alt=""
      draggable={false}
      style={style}
    />
  );
}

/** Background media → lyrics children → foreground media. */
export function MediaStack({
  layout,
  media,
  streamComposite,
  children,
  className,
}: {
  layout: Layout;
  media: MediaAsset[];
  streamComposite?: boolean;
  children?: React.ReactNode;
  className?: string;
}) {
  const layers = layout.mediaLayers ?? [];
  const [urls, setUrls] = useState<Record<string, string>>({});
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  useEffect(() => {
    let cancelled = false;
    (async () => {
      const next: Record<string, string> = {};
      for (const asset of media) {
        if (
          asset.path.startsWith('data:') ||
          asset.path.startsWith('blob:') ||
          asset.path.startsWith('http') ||
          asset.path.startsWith('file:')
        ) {
          next[asset.id] = asset.path;
          continue;
        }
        if (desktop() && !asset.path.startsWith('data:') && !asset.path.startsWith('blob:')) {
          try {
            next[asset.id] = convertFileSrc(asset.path);
          } catch {
            try {
              next[asset.id] = await invoke<string>('media_file_url', { path: asset.path });
            } catch {
              /* keep fallback */
            }
          }
        }
      }
      if (!cancelled) setUrls(next);
    })();
    return () => {
      cancelled = true;
    };
  }, [media]);
  useEffect(() => {
    let cancelled = false;
    (async () => {
      if (desktop()) {
        try {
          const list = await invoke<AudioDevice[]>('list_audio_devices');
          if (!cancelled) setDevices(list);
          return;
        } catch {
          /* fall through */
        }
      }
      try {
        const list = await navigator.mediaDevices.enumerateDevices();
        if (!cancelled) {
          setDevices(
            list
              .filter((d) => d.kind === 'audiooutput')
              .map((d) => ({ id: d.deviceId, label: d.label || 'Audio output' })),
          );
        }
      } catch {
        if (!cancelled) setDevices([]);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);
  const bg = layers.filter((l) => l.role === 'background');
  const fg = layers.filter((l) => l.role === 'foreground');
  const byId = (id: string) => media.find((a) => a.id === id);
  const pump = Boolean(streamComposite);
  return (
    <div className={`media-stack ${className ?? ''}`}>
      {bg.map((layer) => {
        const src = resolveSrc(byId(layer.assetId), urls);
        return src ? (
          <MediaElement
            key={layer.id}
            layout={layout}
            layer={layer}
            src={src}
            pump={pump}
            devices={devices}
          />
        ) : null;
      })}
      <div className="media-lyrics-slot">{children}</div>
      {fg.map((layer) => {
        const src = resolveSrc(byId(layer.assetId), urls);
        return src ? (
          <MediaElement
            key={layer.id}
            layout={layout}
            layer={layer}
            src={src}
            pump={pump}
            devices={devices}
          />
        ) : null;
      })}
    </div>
  );
}
