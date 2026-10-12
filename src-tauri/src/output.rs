use selah_application::OutputPublisher;
use selah_domain::{LiveSnapshot, MediaLayer, MotionSettings};
use selah_infrastructure::ndi::NdiPublisher;
use selah_renderer::{
    compose_stack,
    media::load_rgba,
    transition::{compose, ease, Raster},
    FitMode, Frame, LayerBlit, Renderer, RgbaImage,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStatus {
    pub protocol: String,
    pub sending: bool,
    pub receivers: Option<u32>,
    pub error: Option<String>,
}
/// A transition and/or text animation in flight toward `frame`.
struct Playing {
    live: LiveSnapshot,
    motion: MotionSettings,
    start: Instant,
    from: Option<Raster>,
    to_static: Raster,
    transition_ms: f32,
    text_start_ms: f32,
    text_total_ms: f32,
    last: Option<Raster>,
}
/// Text animation begins this far into a visible transition so the two overlap.
const TEXT_START_FRACTION: f32 = 0.5;
enum Message {
    Live(LiveSnapshot, Sender<Result<(), String>>),
    Toggle(String, bool, String, Sender<Result<(), String>>),
    Status(Sender<Vec<OutputStatus>>),
    MediaFrame(String, u32, u32, Vec<u8>),
    MediaRoot(PathBuf),
}
pub struct Engine {
    tx: Sender<Message>,
}
fn resolve_asset_path(root: &Path, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        root.join(path.trim_start_matches('/'))
    }
}
fn layer_blit(
    layer: &MediaLayer,
    images: &HashMap<String, RgbaImage>,
    frames: &HashMap<String, RgbaImage>,
) -> Option<LayerBlit> {
    let image = if layer.kind == "video" {
        frames.get(&layer.id).cloned()
    } else {
        images
            .get(&layer.asset_id)
            .cloned()
            .or_else(|| frames.get(&layer.id).cloned())
    }?;
    Some(LayerBlit {
        x: layer.x.round() as i32,
        y: layer.y.round() as i32,
        width: layer.width.round().max(1.0) as u32,
        height: layer.height.round().max(1.0) as u32,
        opacity: layer.opacity,
        fit: FitMode::parse(&layer.fit),
        image,
    })
}
fn composite_outgoing(
    live: &LiveSnapshot,
    width: u32,
    height: u32,
    lyrics: &[u8],
    images: &HashMap<String, RgbaImage>,
    frames: &HashMap<String, RgbaImage>,
) -> Option<Vec<u8>> {
    if !live.stream_composite {
        return None;
    }
    let layers = &live.layout.media_layers;
    if layers.is_empty() {
        return None;
    }
    let under: Vec<_> = layers
        .iter()
        .filter(|l| l.role == "background")
        .filter_map(|l| layer_blit(l, images, frames))
        .collect();
    let over: Vec<_> = layers
        .iter()
        .filter(|l| l.role == "foreground")
        .filter_map(|l| layer_blit(l, images, frames))
        .collect();
    if under.is_empty() && over.is_empty() {
        return None;
    }
    compose_stack(width, height, &under, lyrics, &over).ok()
}
fn warm_images(live: &LiveSnapshot, root: &Path, images: &mut HashMap<String, RgbaImage>) {
    for layer in &live.layout.media_layers {
        if layer.kind != "image" || images.contains_key(&layer.asset_id) {
            continue;
        }
        let Some(asset) = live.media.iter().find(|a| a.id == layer.asset_id) else {
            continue;
        };
        let path = resolve_asset_path(root, &asset.path);
        if let Ok(img) = load_rgba(&path) {
            images.insert(layer.asset_id.clone(), img);
        }
    }
}
impl Engine {
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut renderer: Option<Renderer> = None;
            let mut frame: Option<Frame> = None;
            let mut live_snap: Option<LiveSnapshot> = None;
            let mut ndi: Option<NdiPublisher> = None;
            #[cfg(target_os = "macos")]
            let mut syphon: Option<SyphonPublisher> = None;
            let mut errors: [Option<String>; 2] = [None, None];
            let mut playing: Option<Playing> = None;
            let mut last_tick = Instant::now();
            let mut image_cache: HashMap<String, RgbaImage> = HashMap::new();
            let mut video_frames: HashMap<String, RgbaImage> = HashMap::new();
            let mut media_root = PathBuf::new();
            loop {
                match rx.recv_timeout(Duration::from_millis(33).saturating_sub(last_tick.elapsed()))
                {
                    Ok(Message::Live(live, reply)) => {
                        let result: Result<(), String> = (|| {
                            if renderer.is_none() {
                                renderer = Some(Renderer::new()?);
                            }
                            warm_images(&live, &media_root, &mut image_cache);
                            let r = renderer.as_ref().unwrap();
                            let f = r.render(&live)?;
                            let to_static = Raster::from_straight(f.width, f.height, &f.rgba);
                            let motion = live.motion.clone().unwrap_or_default();
                            let from = playing
                                .as_ref()
                                .and_then(|p| p.last.clone())
                                .or_else(|| {
                                    frame
                                        .as_ref()
                                        .map(|o| Raster::from_straight(o.width, o.height, &o.rgba))
                                })
                                .filter(|r| r.width == f.width && r.height == f.height);
                            let transition_ms = if motion.transition.kind != "cut" && from.is_some()
                            {
                                motion.transition.duration_ms as f32
                            } else {
                                0.0
                            };
                            let text_total_ms = Renderer::text_anim_total_ms(&live);
                            frame = Some(f);
                            live_snap = Some(live.clone());
                            playing =
                                (transition_ms > 0.0 || text_total_ms > 0.0).then(|| Playing {
                                    live,
                                    motion,
                                    start: Instant::now(),
                                    from,
                                    to_static,
                                    transition_ms,
                                    text_start_ms: transition_ms * TEXT_START_FRACTION,
                                    text_total_ms,
                                    last: None,
                                });
                            Ok(())
                        })();
                        let _ = reply.send(result);
                    }
                    Ok(Message::Toggle(protocol, enabled, name, reply)) => {
                        let result = match protocol.as_str() {
                            "NDI" => {
                                ndi = None;
                                if enabled {
                                    NdiPublisher::new(&name).map(|p| {
                                        ndi = Some(p);
                                    })
                                } else {
                                    Ok(())
                                }
                            }
                            "Syphon" => {
                                #[cfg(target_os = "macos")]
                                {
                                    syphon = None;
                                    if enabled {
                                        SyphonPublisher::new(&name).map(|p| {
                                            syphon = Some(p);
                                        })
                                    } else {
                                        Ok(())
                                    }
                                }
                                #[cfg(not(target_os = "macos"))]
                                {
                                    Err("Syphon is available only on macOS. Use NDI on Windows."
                                        .into())
                                }
                            }
                            _ => Err("Unknown output protocol".into()),
                        };
                        let i = if protocol == "NDI" { 0 } else { 1 };
                        errors[i] = result.as_ref().err().cloned();
                        let _ = reply.send(result);
                    }
                    Ok(Message::Status(reply)) => {
                        let mut statuses = vec![OutputStatus {
                            protocol: "NDI".into(),
                            sending: ndi.is_some(),
                            receivers: ndi.as_ref().map(|n| n.receivers()),
                            error: errors[0].clone(),
                        }];
                        #[cfg(target_os = "macos")]
                        statuses.push(OutputStatus {
                            protocol: "Syphon".into(),
                            sending: syphon.is_some(),
                            receivers: syphon.as_ref().map(|s| s.receivers()),
                            error: errors[1].clone(),
                        });
                        #[cfg(not(target_os = "macos"))]
                        statuses.push(OutputStatus {
                            protocol: "Syphon".into(),
                            sending: false,
                            receivers: None,
                            error: Some("macOS only".into()),
                        });
                        let _ = reply.send(statuses);
                    }
                    Ok(Message::MediaFrame(layer_id, width, height, rgba)) => {
                        if let Ok(img) = RgbaImage::new(width, height, rgba) {
                            video_frames.insert(layer_id, img);
                        }
                    }
                    Ok(Message::MediaRoot(path)) => {
                        media_root = path;
                        image_cache.clear();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                let mut animated: Option<Raster> = None;
                if let Some(p) = playing.as_mut() {
                    let el = p.start.elapsed().as_secs_f32() * 1000.0;
                    let text_ms = el - p.text_start_ms;
                    let to = if p.text_total_ms > 0.0 && text_ms < p.text_total_ms {
                        renderer
                            .as_ref()
                            .and_then(|r| r.render_raster(&p.live, Some(text_ms.max(0.0))).ok())
                            .unwrap_or_else(|| p.to_static.clone())
                    } else {
                        p.to_static.clone()
                    };
                    let spec = &p.motion.transition;
                    let out = match (&p.from, el < p.transition_ms) {
                        (Some(from), true) => {
                            compose(from, &to, spec, ease(&spec.easing, el / p.transition_ms))
                        }
                        _ => to,
                    };
                    if el >= p.transition_ms.max(p.text_start_ms + p.text_total_ms) {
                        playing = None; // settled: the final frame below takes over.
                    } else {
                        p.last = Some(out.clone());
                        animated = Some(out);
                    }
                }
                let straight = animated
                    .as_ref()
                    .map(|r| (r.width, r.height, r.to_straight()));
                let owned_composite: Option<(u32, u32, Vec<u8>)> = {
                    let lyrics = straight
                        .as_ref()
                        .map(|(w, h, d)| (*w, *h, d.as_slice()))
                        .or(frame
                            .as_ref()
                            .map(|f| (f.width, f.height, f.rgba.as_slice())));
                    if let (Some((w, h, rgba)), Some(live)) = (lyrics, live_snap.as_ref()) {
                        composite_outgoing(live, w, h, rgba, &image_cache, &video_frames)
                            .map(|buf| (w, h, buf))
                    } else {
                        None
                    }
                };
                let outgoing = owned_composite
                    .as_ref()
                    .map(|(w, h, d)| (*w, *h, d.as_slice()))
                    .or_else(|| {
                        straight
                            .as_ref()
                            .map(|(w, h, d)| (*w, *h, d.as_slice()))
                            .or(frame
                                .as_ref()
                                .map(|f| (f.width, f.height, f.rgba.as_slice())))
                    });
                if let Some((width, height, rgba)) = outgoing {
                    if let Some(n) = ndi.as_mut() {
                        if let Err(e) = n.publish(width, height, rgba) {
                            errors[0] = Some(e);
                            ndi = None;
                        }
                    }
                    #[cfg(target_os = "macos")]
                    if let Some(s) = syphon.as_mut() {
                        if let Err(e) = s.publish(width, height, rgba) {
                            errors[1] = Some(e);
                            syphon = None;
                        }
                    }
                }
                last_tick = Instant::now();
            }
        });
        Self { tx }
    }
    pub fn live(&self, live: LiveSnapshot) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Message::Live(live, tx))
            .map_err(|_| "Output worker stopped")?;
        rx.recv_timeout(Duration::from_secs(15))
            .map_err(|_| "Renderer timed out")?
    }
    pub fn toggle(&self, protocol: String, enabled: bool, name: String) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Message::Toggle(protocol, enabled, name, tx))
            .map_err(|_| "Output worker stopped")?;
        rx.recv_timeout(Duration::from_secs(15))
            .map_err(|_| "Output setup timed out")?
    }
    pub fn status(&self) -> Result<Vec<OutputStatus>, String> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Message::Status(tx))
            .map_err(|_| "Output worker stopped")?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| "Status timed out".into())
    }
    pub fn push_media_frame(
        &self,
        layer_id: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<(), String> {
        self.tx
            .send(Message::MediaFrame(layer_id, width, height, rgba))
            .map_err(|_| "Output worker stopped".into())
    }
    pub fn set_media_root(&self, path: PathBuf) -> Result<(), String> {
        self.tx
            .send(Message::MediaRoot(path))
            .map_err(|_| "Output worker stopped".into())
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn selah_syphon_create(name: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn selah_syphon_publish(
        handle: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        rgba: *const u8,
    ) -> i32;
    fn selah_syphon_clients(handle: *mut std::ffi::c_void) -> i32;
    fn selah_syphon_destroy(handle: *mut std::ffi::c_void);
}
#[cfg(target_os = "macos")]
struct SyphonPublisher(*mut std::ffi::c_void);
#[cfg(target_os = "macos")]
impl SyphonPublisher {
    fn new(name: &str) -> Result<Self, String> {
        let name = std::ffi::CString::new(name).map_err(|_| "Invalid feed name")?;
        let h = unsafe { selah_syphon_create(name.as_ptr()) };
        if h.is_null() {
            Err("Install Syphon.framework in /Library/Frameworks, then restart Selah".into())
        } else {
            Ok(Self(h))
        }
    }
    fn receivers(&self) -> u32 {
        (unsafe { selah_syphon_clients(self.0) }) as u32
    }
}
#[cfg(target_os = "macos")]
impl OutputPublisher for SyphonPublisher {
    fn publish(&mut self, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
        if unsafe { selah_syphon_publish(self.0, w, h, rgba.as_ptr()) } == 0 {
            Err("Syphon could not publish frame".into())
        } else {
            Ok(())
        }
    }
}
#[cfg(target_os = "macos")]
impl Drop for SyphonPublisher {
    fn drop(&mut self) {
        unsafe { selah_syphon_destroy(self.0) }
    }
}
