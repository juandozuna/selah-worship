//! Physical full-screen output: a borderless window on a chosen monitor with its own layout and background.
use super::*;
use selah_domain::{DisplayOutput, LiveSnapshot};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "display";

#[derive(Default)]
pub struct DisplayState {
    revision: AtomicU64,
    live: Mutex<Option<LiveSnapshot>>,
    config: Mutex<Option<DisplayOutput>>,
}
impl DisplayState {
    pub fn set_live(&self, live: LiveSnapshot) {
        if let Ok(mut lock) = self.live.lock() {
            *lock = Some(live);
        }
        self.revision.fetch_add(1, Ordering::SeqCst);
    }
    fn set_config(&self, config: DisplayOutput) {
        if let Ok(mut lock) = self.config.lock() {
            *lock = Some(config);
        }
        self.revision.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    index: usize,
    name: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f64,
    primary: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOverview {
    displays: Vec<DisplayInfo>,
    open: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayFrame {
    revision: u64,
    live: Option<LiveSnapshot>,
    config: Option<DisplayOutput>,
}

#[tauri::command]
pub async fn list_displays(app: AppHandle) -> Result<DisplayOverview, String> {
    let primary = app.primary_monitor().map_err(|e| e.to_string())?;
    let displays = app
        .available_monitors()
        .map_err(|e| e.to_string())?
        .iter()
        .enumerate()
        .map(|(index, m)| DisplayInfo {
            index,
            name: m
                .name()
                .cloned()
                .unwrap_or_else(|| format!("Display {}", index + 1)),
            x: m.position().x,
            y: m.position().y,
            width: m.size().width,
            height: m.size().height,
            scale: m.scale_factor(),
            primary: primary
                .as_ref()
                .is_some_and(|p| p.position() == m.position() && p.size() == m.size()),
        })
        .collect();
    Ok(DisplayOverview {
        displays,
        open: app.get_webview_window(LABEL).is_some(),
    })
}

#[tauri::command]
pub async fn open_display(
    app: AppHandle,
    state: State<'_, AppState>,
    display: DisplayOutput,
) -> Result<(), String> {
    display.validate()?;
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .get(display.monitor)
        .ok_or("That display is no longer connected")?;
    let position = PhysicalPosition::new(monitor.position().x, monitor.position().y);
    state.display.set_config(display);
    let window = match app.get_webview_window(LABEL) {
        Some(w) => {
            let _ = w.set_fullscreen(false);
            w
        }
        None => {
            WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("index.html?display=1".into()))
                .title("Selah Display")
                .decorations(false)
                .resizable(false)
                .build()
                .map_err(|e| e.to_string())?
        }
    };
    window.set_position(position).map_err(|e| e.to_string())?;
    window.set_fullscreen(true).map_err(|e| e.to_string())?;
    let _ = window.set_cursor_visible(false);
    Ok(())
}

/// Applies a new layout/background to an open display without moving the window.
#[tauri::command]
pub async fn update_display(
    app: AppHandle,
    state: State<'_, AppState>,
    display: DisplayOutput,
) -> Result<(), String> {
    display.validate()?;
    if app.get_webview_window(LABEL).is_some() {
        state.display.set_config(display);
    }
    Ok(())
}

#[tauri::command]
pub async fn close_display(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(LABEL) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn display_frame(state: State<'_, AppState>) -> DisplayFrame {
    DisplayFrame {
        revision: state.display.revision.load(Ordering::SeqCst),
        live: state.display.live.lock().ok().and_then(|l| l.clone()),
        config: state.display.config.lock().ok().and_then(|c| c.clone()),
    }
}
