#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod display;
mod output;
mod remote;
use base64::Engine as _;
use selah_application::{BibleRepository, CredentialStore};
use selah_domain::*;
use selah_infrastructure::{
    ai::LocalWorker, credentials::SystemCredentials, imports, network, process,
    storage::SqliteStore,
};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{Manager, State};

struct AppState {
    store: Arc<Mutex<SqliteStore>>,
    output: Arc<output::Engine>,
    worker: Arc<Mutex<Option<LocalWorker>>>,
    data_dir: PathBuf,
    runtime_path: PathBuf,
    preview_renderer: Arc<Mutex<Option<selah_renderer::Renderer>>>,
    display: Arc<display::DisplayState>,
}
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn load_project(state: State<'_, AppState>, mut seed: Project) -> Result<Project, String> {
    let store = state.store.clone();
    let output = state.output.clone();
    let display = state.display.clone();
    let media_dir = state.data_dir.join("media");
    seed.settings.local_server_path = state.runtime_path.to_string_lossy().into_owned();
    let model = state.data_dir.join("qwen3-4b-q4.gguf");
    if model.is_file() {
        seed.settings.local_model_path = model.to_string_lossy().into_owned();
    }
    let runtime = seed.settings.local_server_path.clone();
    blocking(move || {
        let mut p = selah_application::load_project(
            &*store.lock().map_err(|_| "Database unavailable")?,
            seed,
        )?;
        if p.settings.local_server_path.is_empty() {
            p.settings.local_server_path = runtime;
        }
        if p.settings.local_model_path.is_empty() && model.is_file() {
            p.settings.local_model_path = model.to_string_lossy().into_owned();
        }
        std::fs::create_dir_all(&media_dir).map_err(|e| e.to_string())?;
        let _ = output.set_media_root(media_dir);
        let cleared = selah_application::clear_with_media(&p);
        display.set_live(cleared.clone());
        output.live(cleared)?;
        Ok(p)
    })
    .await
}
#[tauri::command]
async fn save_project(state: State<'_, AppState>, project: Project) -> Result<(), String> {
    let store = state.store.clone();
    blocking(move || {
        selah_application::save_project(
            &*store.lock().map_err(|_| "Database unavailable")?,
            &project,
        )
    })
    .await
}
#[tauri::command]
async fn set_live(state: State<'_, AppState>, live: LiveSnapshot) -> Result<(), String> {
    let output = state.output.clone();
    let display = state.display.clone();
    blocking(move || {
        display.set_live(live.clone());
        output.live(live)
    })
    .await
}
/// Preparation only: callers must serialize this and publish the returned intent
/// through the existing present/set_live queue. No adapter or clock runs here.
#[tauri::command]
async fn apply_presentation_command(
    project: Project,
    current: LiveSnapshot,
    status: selah_application::presenter::ControlStatus,
    command: PresentationCommand,
    origin: CommandOrigin,
    selected_song_id: Option<String>,
    motion_on: bool,
) -> Result<selah_application::presenter::CommandIntent, String> {
    blocking(move || {
        selah_application::presenter::apply_presentation_command(
            &project,
            &current,
            &status,
            &command,
            origin,
            selected_song_id.as_deref(),
            motion_on,
        )
    })
    .await
}

/// Reports capabilities, not a running chase state: Wave 1 has no listeners.
#[tauri::command]
fn control_status() -> serde_json::Value {
    serde_json::json!({"commandApiVersion": 1, "midiListening": false, "oscListening": false,
        "mtcAvailable": false, "ltcAvailable": false, "timelineClockRunning": false})
}

#[tauri::command]
async fn render_preview(state: State<'_, AppState>, live: LiveSnapshot) -> Result<String, String> {
    let renderer = state.preview_renderer.clone();
    blocking(move || {
        let mut lock = renderer
            .lock()
            .map_err(|_| "Preview renderer unavailable")?;
        if lock.is_none() {
            *lock = Some(selah_renderer::Renderer::new()?);
        }
        let frame = lock.as_ref().unwrap().render(&live)?;
        Ok(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(frame.png)
        ))
    })
    .await
}

#[tauri::command]
async fn toggle_output(
    state: State<'_, AppState>,
    protocol: String,
    enabled: bool,
    feed_name: String,
) -> Result<(), String> {
    let output = state.output.clone();
    blocking(move || {
        if feed_name.trim().is_empty() || feed_name.len() > 100 {
            return Err("Use a feed name between 1 and 100 characters".into());
        }
        output.toggle(protocol, enabled, feed_name)
    })
    .await
}
#[tauri::command]
async fn output_status(state: State<'_, AppState>) -> Result<Vec<output::OutputStatus>, String> {
    let output = state.output.clone();
    blocking(move || output.status()).await
}
#[tauri::command]
async fn save_credential(provider: String, value: String) -> Result<(), String> {
    blocking(move || SystemCredentials.set(&provider, &value)).await
}
#[tauri::command]
async fn import_openlyrics(text: String) -> Result<Vec<PresentationBlock>, String> {
    blocking(move || imports::openlyrics(&text)).await
}
#[tauri::command]
async fn bible_translations(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let store = state.store.clone();
    blocking(move || {
        store
            .lock()
            .map_err(|_| "Database unavailable")?
            .translations()
    })
    .await
}
#[tauri::command]
async fn bible_lookup(
    state: State<'_, AppState>,
    reference: String,
    translation: String,
) -> Result<Vec<BibleVerse>, String> {
    let store = state.store.clone();
    blocking(move || {
        let passage = selah_application::parse_passage(&reference)?;
        store
            .lock()
            .map_err(|_| "Database unavailable")?
            .verses(&translation, &passage)
    })
    .await
}

fn parse_bible_package(
    bytes: Vec<u8>,
    name: &str,
    translation: &str,
) -> Result<Vec<BibleVerse>, String> {
    if translation.is_empty() || translation.len() > 50 || bytes.len() > 20_971_520 {
        return Err("Invalid translation name or oversized package".into());
    }
    fn text_file(text: &str, name: &str, t: &str) -> Result<Vec<BibleVerse>, String> {
        if name.ends_with(".usfm") || name.ends_with(".sfm") {
            imports::usfm(text, t)
        } else {
            imports::vpl(text, t)
        }
    }
    if name.ends_with(".zip") {
        let mut zip =
            zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let mut verses = Vec::new();
        let mut total = 0u64;
        if zip.len() > 200 {
            return Err("Too many files in Bible package".into());
        }
        for i in 0..zip.len() {
            let file = zip.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            if ![".txt", ".vpl", ".usfm", ".sfm"]
                .iter()
                .any(|e| name.ends_with(e))
            {
                continue;
            }
            if name.contains("readme") || name.contains("README") {
                continue;
            }
            total += file.size();
            if total > 50_000_000 {
                return Err("Bible archive is too large".into());
            }
            let mut text = String::new();
            file.take(50_000_001)
                .read_to_string(&mut text)
                .map_err(|_| "Bible file is not UTF-8")?;
            if let Ok(mut v) = text_file(&text, &name, translation) {
                verses.append(&mut v);
            }
        }
        if verses.is_empty() {
            return Err("No supported verses found in package".into());
        }
        Ok(verses)
    } else {
        text_file(
            std::str::from_utf8(&bytes).map_err(|_| "Bible file is not UTF-8")?,
            name,
            translation,
        )
    }
}
#[tauri::command]
async fn import_bible(
    state: State<'_, AppState>,
    bytes: Vec<u8>,
    name: String,
    translation: String,
) -> Result<usize, String> {
    let store = state.store.clone();
    blocking(move || {
        let verses = parse_bible_package(bytes, &name, &translation)?;
        store
            .lock()
            .map_err(|_| "Database unavailable")?
            .import_verses(&verses)
    })
    .await
}
#[tauri::command]
async fn download_bible(state: State<'_, AppState>, translation: String) -> Result<usize, String> {
    let store = state.store.clone();
    blocking(move || {
        let id = match translation.as_str() {
            "WEB" => "engwebp",
            "RV1909" => "spaRV1909",
            _ => return Err("Download supports WEB and RV1909; import other translations".into()),
        };
        let name = format!("{id}_vpl.zip");
        let bytes = network::request(
            &format!("https://ebible.org/Scriptures/{name}"),
            &[],
            None,
            false,
        )?;
        let verses = parse_bible_package(bytes, &name, &translation)?;
        store
            .lock()
            .map_err(|_| "Database unavailable")?
            .import_verses(&verses)
    })
    .await
}
#[tauri::command]
async fn search_lyrics(query: String) -> Result<Vec<SearchCandidate>, String> {
    blocking(move || network::search(&query, &SystemCredentials.get("brave")?)).await
}
#[tauri::command]
async fn extract_lyrics(url: String) -> Result<String, String> {
    blocking(move || network::extract(&url)).await
}
#[tauri::command]
async fn analyze_lyrics(
    state: State<'_, AppState>,
    text: String,
    settings: Settings,
    lines_per_block: Option<usize>,
) -> Result<Vec<PresentationBlock>, String> {
    let worker = state.worker.clone();
    let dir = state.data_dir.clone();
    let runtime = state.runtime_path.clone();
    blocking(move || {
        let settings = if settings.ai_provider == "local" {
            prepare_local_settings(&dir, &runtime, settings)?
        } else {
            settings
        };
        if settings.ai_provider == "local" {
            ensure_local_worker(&worker, &settings)?;
        }
        let key = if settings.ai_provider == "local" {
            String::new()
        } else {
            SystemCredentials.get(&settings.ai_provider)?
        };
        selah_infrastructure::ai::analyze_with_lines(
            &text,
            &settings,
            &key,
            lines_per_block.unwrap_or(2),
        )
    })
    .await
}
#[tauri::command]
async fn prepare_web_lyrics(
    state: State<'_, AppState>,
    text: String,
    title: String,
    lines_per_block: usize,
    mut settings: Settings,
) -> Result<serde_json::Value, String> {
    settings.ai_provider = "local".into();
    let worker = state.worker.clone();
    let dir = state.data_dir.clone();
    let runtime = state.runtime_path.clone();
    blocking(move || {
        if !(1..=8).contains(&lines_per_block) {
            return Err("Choose 1–8 lines per block".into());
        }
        let settings = prepare_local_settings(&dir, &runtime, settings)?;
        ensure_local_worker(&worker, &settings)?;
        let text = selah_infrastructure::ai::extract_lyric_text_local(&text, &title)?;
        let blocks =
            selah_infrastructure::ai::analyze_with_lines(&text, &settings, "", lines_per_block)?;
        Ok(serde_json::json!({"text":text,"blocks":blocks}))
    })
    .await
}

#[tauri::command]
async fn translate_lyrics(
    state: State<'_, AppState>,
    text: String,
    target: String,
    mut settings: Settings,
) -> Result<String, String> {
    // Translation is explicitly local, even when organization uses a paid provider.
    settings.ai_provider = "local".into();
    let worker = state.worker.clone();
    let dir = state.data_dir.clone();
    let runtime = state.runtime_path.clone();
    blocking(move || {
        let settings = prepare_local_settings(&dir, &runtime, settings)?;
        ensure_local_worker(&worker, &settings)?;
        selah_infrastructure::ai::translate_local(&text, &target)
    })
    .await
}

#[tauri::command]
async fn test_ai(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let worker = state.worker.clone();
    let dir = state.data_dir.clone();
    let runtime = state.runtime_path.clone();
    blocking(move || {
        let settings = if settings.ai_provider == "local" {
            prepare_local_settings(&dir, &runtime, settings)?
        } else {
            settings
        };
        if settings.ai_provider == "local" {
            ensure_local_worker(&worker, &settings)?;
        }
        let key = if settings.ai_provider == "local" {
            String::new()
        } else {
            SystemCredentials.get(&settings.ai_provider)?
        };
        selah_infrastructure::ai::analyze("Santo es el Señor\nHoly is the Lord", &settings, &key)
            .map(|_| ())
    })
    .await
}
fn ensure_local_worker(
    worker: &Arc<Mutex<Option<LocalWorker>>>,
    settings: &Settings,
) -> Result<(), String> {
    let mut lock = worker.lock().map_err(|_| "AI worker unavailable")?;
    if !lock.as_mut().is_some_and(|w| w.is_running()) {
        *lock = Some(LocalWorker::start(settings)?);
    }
    drop(lock);
    for _ in 0..60 {
        if !worker
            .lock()
            .map_err(|_| "AI worker unavailable")?
            .as_mut()
            .is_some_and(|w| w.is_running())
        {
            return Err("Local AI exited. Set up the model in Settings.".into());
        }
        if network::request("http://127.0.0.1:47831/health", &[], None, true).is_ok() {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Err("The local model is still loading. Try again in a moment.".into())
}

#[tauri::command]
fn model_download_progress(state: State<'_, AppState>) -> u64 {
    std::fs::metadata(state.data_dir.join("qwen3-4b-q4.gguf.download"))
        .map(|m| m.len())
        .unwrap_or(0)
}

#[tauri::command]
async fn start_local_ai(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let worker = state.worker.clone();
    let dir = state.data_dir.clone();
    let runtime = state.runtime_path.clone();
    blocking(move || {
        let settings = prepare_local_settings(&dir, &runtime, settings)?;
        *worker.lock().map_err(|_| "AI worker unavailable")? = None;
        ensure_local_worker(&worker, &settings)
    })
    .await
}

#[tauri::command]
async fn setup_local_ai(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let worker = state.worker.clone();
    let dir = state.data_dir.clone();
    let runtime = state.runtime_path.clone();
    blocking(move || {
        let settings = prepare_local_settings(&dir, &runtime, settings)?;
        ensure_local_worker(&worker, &settings)?;
        Ok(settings)
    })
    .await
}

fn prepare_local_settings(
    dir: &std::path::Path,
    runtime: &std::path::Path,
    mut settings: Settings,
) -> Result<Settings, String> {
    if settings.local_server_path.is_empty() {
        settings.local_server_path = runtime.to_string_lossy().into_owned();
    }
    if !std::path::Path::new(&settings.local_server_path).is_file() {
        return Err("The bundled AI runtime is missing. Reinstall Selah, or select llama-server in Advanced local settings.".into());
    }
    if settings.local_model_path.is_empty() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../assets/model.json"))
                .map_err(|e| e.to_string())?;
        let installed = dir.join("qwen3-4b-q4.gguf");
        if installed.is_file() {
            settings.local_model_path = installed.to_string_lossy().into_owned();
            return Ok(settings);
        }
        settings.local_model_path = install_model(
            dir,
            manifest["url"].as_str().ok_or("Missing model URL")?,
            manifest["sha256"]
                .as_str()
                .ok_or("Missing model checksum")?,
        )?;
    }
    Ok(settings)
}

#[tauri::command]
async fn download_model(
    state: State<'_, AppState>,
    url: String,
    checksum: String,
) -> Result<String, String> {
    let dir = state.data_dir.clone();
    blocking(move || install_model(&dir, &url, &checksum)).await
}

static MODEL_INSTALL: Mutex<()> = Mutex::new(());
fn install_model(dir: &std::path::Path, url: &str, checksum: &str) -> Result<String, String> {
    let _install = MODEL_INSTALL
        .lock()
        .map_err(|_| "Model installer unavailable")?;
    if process::is_shutting_down() {
        return Err("Selah is shutting down".into());
    }
    if checksum.len() != 64 || !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Enter the publisher's SHA-256".into());
    }
    let path = dir.join("qwen3-4b-q4.gguf.download");
    let final_path = dir.join("qwen3-4b-q4.gguf");
    if final_path.is_file() && verified_model(&final_path, checksum)? {
        return Ok(final_path.to_string_lossy().into_owned());
    }
    // Recover a completed download from a previous interrupted setup before using the network.
    if !path.is_file() || !verified_model(&path, checksum)? {
        let (url, target) = network::resolve_download(url)?;
        let child = process::ManagedProcess::spawn(
            std::process::Command::new("curl")
                .args([
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--noproxy",
                    "*",
                    "--resolve",
                    &target,
                    "--connect-timeout",
                    "30",
                    "--max-time",
                    "3600",
                    "--proto",
                    "=https",
                    "--output",
                ])
                .arg(&path)
                .arg(&url)
                .stdin(std::process::Stdio::null()),
        )
        .map_err(|e| e.to_string())?;
        if !child.wait().map_err(|e| e.to_string())?.success() {
            return Err(
                "Model download interrupted. Retry local AI setup when connected to the internet."
                    .into(),
            );
        }
    }
    if !verified_model(&path, checksum)? {
        let _ = std::fs::remove_file(&path);
        return Err("Model checksum did not match. The download was discarded.".into());
    }
    if process::is_shutting_down() {
        return Err("Selah is shutting down".into());
    }
    std::fs::rename(&path, &final_path).map_err(|e| e.to_string())?;
    Ok(final_path.to_string_lossy().into_owned())
}

fn verified_model(path: &std::path::Path, checksum: &str) -> Result<bool, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        if process::is_shutting_down() {
            return Err("Selah is shutting down".into());
        }
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    if format!("{:x}", hasher.finalize()) != checksum.to_lowercase() {
        return Ok(false);
    }
    let mut magic = [0u8; 4];
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .read_exact(&mut magic)
        .map_err(|e| e.to_string())?;
    Ok(&magic == b"GGUF")
}

#[tauri::command]
fn remote_urls() -> Vec<String> {
    remote::urls()
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioDeviceInfo {
    id: String,
    label: String,
}

#[tauri::command]
fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let mut out = Vec::new();
    let devices = host
        .output_devices()
        .map_err(|e| format!("Could not list audio outputs: {e}"))?;
    for device in devices {
        let label = device
            .name()
            .unwrap_or_else(|_| "Audio output".into());
        out.push(AudioDeviceInfo {
            id: label.clone(),
            label,
        });
    }
    if out.is_empty() {
        out.push(AudioDeviceInfo {
            id: "default".into(),
            label: "System default".into(),
        });
    }
    Ok(out)
}

#[tauri::command]
async fn import_media_file(state: State<'_, AppState>) -> Result<MediaAsset, String> {
    let media_dir = state.data_dir.join("media");
    blocking(move || {
        std::fs::create_dir_all(&media_dir).map_err(|e| e.to_string())?;
        let file = rfd::FileDialog::new()
            .add_filter(
                "Media",
                &[
                    "png", "jpg", "jpeg", "webp", "gif", "mp4", "mov", "webm", "m4v",
                ],
            )
            .pick_file()
            .ok_or_else(|| "No file selected".to_string())?;
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
            .to_lowercase();
        let kind = if matches!(
            ext.as_str(),
            "mp4" | "mov" | "webm" | "m4v" | "avi" | "mkv"
        ) {
            "video"
        } else {
            "image"
        };
        let mime = match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "mp4" | "m4v" => "video/mp4",
            "mov" => "video/quicktime",
            "webm" => "video/webm",
            _ => "application/octet-stream",
        };
        let id = format!(
            "media-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("media")
            .to_string();
        let dest_name = format!("{id}.{ext}");
        let dest = media_dir.join(&dest_name);
        std::fs::copy(&file, &dest).map_err(|e| format!("Could not import media: {e}"))?;
        Ok(MediaAsset {
            id,
            name,
            path: dest.to_string_lossy().into_owned(),
            kind: kind.into(),
            mime: mime.into(),
        })
    })
    .await
}

#[tauri::command]
async fn push_media_frame(
    state: State<'_, AppState>,
    layer_id: String,
    width: u32,
    height: u32,
    rgba_base64: String,
) -> Result<(), String> {
    let output = state.output.clone();
    blocking(move || {
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return Err("Invalid media frame size".into());
        }
        let rgba = base64::engine::general_purpose::STANDARD
            .decode(rgba_base64.as_bytes())
            .map_err(|e| format!("Invalid media frame: {e}"))?;
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err("Media frame size mismatch".into());
        }
        output.push_media_frame(layer_id, width, height, rgba)
    })
    .await
}

#[tauri::command]
fn media_file_url(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.is_file() {
        return Err("Media file not found".into());
    }
    Ok(format!(
        "file://{}",
        p.to_string_lossy().replace(' ', "%20")
    ))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let mut store =
                SqliteStore::open(&dir.join("selah.sqlite")).map_err(std::io::Error::other)?;
            let resources = app.path().resource_dir()?;
            let translations = store.translations().map_err(std::io::Error::other)?;
            for (id, translation) in [("engwebp", "WEB"), ("spaRV1909", "RV1909")] {
                let name = format!("{id}_vpl.zip");
                let path = resources.join("bibles").join(&name);
                if !translations.iter().any(|t| t == translation) && path.is_file() {
                    let verses = parse_bible_package(std::fs::read(path)?, &name, translation)
                        .map_err(std::io::Error::other)?;
                    if verses.len() < 30000 {
                        return Err(
                            std::io::Error::other("Bundled Bible package is incomplete").into()
                        );
                    }
                    store
                        .import_verses(&verses)
                        .map_err(std::io::Error::other)?;
                }
            }
            let executable = if cfg!(target_os = "windows") {
                "llama-server.exe"
            } else {
                "llama-server"
            };
            let media_dir = dir.join("media");
            std::fs::create_dir_all(&media_dir)?;
            let engine = output::Engine::start();
            let _ = engine.set_media_root(media_dir);
            app.manage(AppState {
                store: Arc::new(Mutex::new(store)),
                output: Arc::new(engine),
                worker: Arc::new(Mutex::new(None)),
                data_dir: dir,
                runtime_path: resources.join("runtime").join(executable),
                preview_renderer: Arc::new(Mutex::new(None)),
                display: Arc::new(display::DisplayState::default()),
            });
            remote::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_project,
            save_project,
            set_live,
            apply_presentation_command,
            control_status,
            render_preview,
            toggle_output,
            output_status,
            save_credential,
            import_openlyrics,
            bible_translations,
            bible_lookup,
            import_bible,
            download_bible,
            search_lyrics,
            extract_lyrics,
            analyze_lyrics,
            prepare_web_lyrics,
            translate_lyrics,
            test_ai,
            start_local_ai,
            setup_local_ai,
            download_model,
            model_download_progress,
            remote_urls,
            list_audio_devices,
            import_media_file,
            push_media_frame,
            media_file_url,
            display::list_displays,
            display::open_display,
            display::update_display,
            display::close_display,
            display::display_frame
        ])
        .build(tauri::generate_context!())
        .expect("Could not launch Selah")
        .run(|app, event| match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { .. },
                ..
            } if label == "main" => {
                process::shutdown();
                app.exit(0);
            }
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => process::shutdown(),
            _ => {}
        });
}

#[cfg(test)]
mod local_install_tests {
    use super::*;
    static FIXTURE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    fn fixture() -> (PathBuf, String) {
        let dir = std::env::temp_dir().join(format!(
            "selah-model-test-{}-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let bytes = b"GGUF test model";
        std::fs::write(dir.join("qwen3-4b-q4.gguf.download"), bytes).unwrap();
        (dir, format!("{:x}", Sha256::digest(bytes)))
    }
    #[test]
    fn completed_download_is_verified_and_recovered_without_network() {
        let (dir, checksum) = fixture();
        let installed = install_model(&dir, "https://invalid.example/model", &checksum).unwrap();
        assert!(std::path::Path::new(&installed).is_file());
        assert!(!dir.join("qwen3-4b-q4.gguf.download").exists());
        assert_eq!(
            install_model(&dir, "https://invalid.example/model", &checksum).unwrap(),
            installed
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn digest_and_gguf_signature_are_both_required() {
        let (dir, checksum) = fixture();
        let path = dir.join("qwen3-4b-q4.gguf.download");
        assert!(verified_model(&path, &checksum).unwrap());
        assert!(!verified_model(&path, &"0".repeat(64)).unwrap());
        std::fs::write(&path, b"not a model").unwrap();
        let checksum = format!("{:x}", Sha256::digest(b"not a model"));
        assert!(!verified_model(&path, &checksum).unwrap());
        assert!(install_model(&dir, "invalid", "invalid").is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
