//! Built-in web server: while the desktop app runs, the same UI and commands are reachable from any device on the network.
use super::*;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io::Read as _;
use tiny_http::{Header, Method, Response, Server};

pub const PORT: u16 = 47832;
const MAX_BODY: u64 = 100 * 1024 * 1024;
/// Commands that touch credentials, local runtimes or large downloads stay desktop-only.
const DESKTOP_ONLY: [&str; 4] = [
    "save_credential",
    "download_model",
    "start_local_ai",
    "setup_local_ai",
];

fn arg<T: DeserializeOwned>(args: &Value, key: &str) -> Result<T, String> {
    serde_json::from_value(args.get(key).cloned().unwrap_or(Value::Null))
        .map_err(|e| format!("Invalid '{key}': {e}"))
}
fn json<T: serde::Serialize>(v: T) -> Result<Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

pub fn lan_address() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:9").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
pub fn urls() -> Vec<String> {
    let mut urls: Vec<String> = lan_address()
        .map(|ip| format!("http://{ip}:{PORT}"))
        .into_iter()
        .collect();
    urls.push(format!("http://localhost:{PORT}"));
    urls
}

async fn dispatch(app: &tauri::AppHandle, cmd: &str, a: Value) -> Result<Value, String> {
    if DESKTOP_ONLY.contains(&cmd) {
        return Err("This action is only available in the desktop app.".into());
    }
    match cmd {
        "load_project" => json(load_project(app.state(), arg(&a, "seed")?).await?),
        "save_project" => json(save_project(app.state(), arg(&a, "project")?).await?),
        "set_live" => json(set_live(app.state(), arg(&a, "live")?).await?),
        "render_preview" => json(render_preview(app.state(), arg(&a, "live")?).await?),
        "toggle_output" => json(
            toggle_output(
                app.state(),
                arg(&a, "protocol")?,
                arg(&a, "enabled")?,
                arg(&a, "feedName")?,
            )
            .await?,
        ),
        "output_status" => json(output_status(app.state()).await?),
        "import_openlyrics" => json(import_openlyrics(arg(&a, "text")?).await?),
        "bible_translations" => json(bible_translations(app.state()).await?),
        "bible_lookup" => {
            json(bible_lookup(app.state(), arg(&a, "reference")?, arg(&a, "translation")?).await?)
        }
        "import_bible" => json(
            import_bible(
                app.state(),
                arg(&a, "bytes")?,
                arg(&a, "name")?,
                arg(&a, "translation")?,
            )
            .await?,
        ),
        "download_bible" => json(download_bible(app.state(), arg(&a, "translation")?).await?),
        "search_lyrics" => json(search_lyrics(arg(&a, "query")?).await?),
        "extract_lyrics" => json(extract_lyrics(arg(&a, "url")?).await?),
        "analyze_lyrics" => json(
            analyze_lyrics(
                app.state(),
                arg(&a, "text")?,
                arg(&a, "settings")?,
                a.get("linesPerBlock")
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()
                    .map_err(|e| e.to_string())?,
            )
            .await?,
        ),
        "prepare_web_lyrics" => json(
            prepare_web_lyrics(
                app.state(),
                arg(&a, "text")?,
                arg(&a, "title")?,
                arg(&a, "linesPerBlock")?,
                arg(&a, "settings")?,
            )
            .await?,
        ),
        "translate_lyrics" => json(
            translate_lyrics(
                app.state(),
                arg(&a, "text")?,
                arg(&a, "target")?,
                arg(&a, "settings")?,
            )
            .await?,
        ),
        "test_ai" => json(test_ai(app.state(), arg(&a, "settings")?).await?),
        "model_download_progress" => json(model_download_progress(app.state())),
        "remote_urls" => json(urls()),
        "list_displays" => json(display::list_displays(app.clone()).await?),
        "open_display" => {
            json(display::open_display(app.clone(), app.state(), arg(&a, "display")?).await?)
        }
        "update_display" => {
            json(display::update_display(app.clone(), app.state(), arg(&a, "display")?).await?)
        }
        "close_display" => json(display::close_display(app.clone()).await?),
        _ => Err(format!("Unknown command: {cmd}")),
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}
fn reply(
    request: tiny_http::Request,
    status: u16,
    content_type: &str,
    body: Vec<u8>,
    csp: Option<String>,
) {
    let mut response = Response::from_data(body)
        .with_status_code(status)
        .with_header(header("Content-Type", content_type))
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("Cache-Control", "no-store"));
    if let Some(csp) = csp {
        response = response.with_header(header("Content-Security-Policy", &csp));
    }
    let _ = request.respond(response);
}

fn handle(app: &tauri::AppHandle, mut request: tiny_http::Request) {
    let url = request.url().split('?').next().unwrap_or("/").to_string();
    if request.method() == &Method::Post {
        let Some(cmd) = url.strip_prefix("/api/invoke/") else {
            return reply(request, 404, "text/plain", b"Not found".to_vec(), None);
        };
        let is_json = request.headers().iter().any(|h| {
            h.field.equiv("Content-Type") && h.value.as_str().starts_with("application/json")
        });
        if !is_json {
            return reply(request, 415, "text/plain", b"JSON required".to_vec(), None);
        }
        let mut body = Vec::new();
        if request
            .as_reader()
            .take(MAX_BODY + 1)
            .read_to_end(&mut body)
            .is_err()
            || body.len() as u64 > MAX_BODY
        {
            return reply(
                request,
                413,
                "text/plain",
                b"Request too large".to_vec(),
                None,
            );
        }
        let result = serde_json::from_slice::<Value>(&body)
            .map_err(|e| e.to_string())
            .and_then(|args| tauri::async_runtime::block_on(dispatch(app, cmd, args)));
        let (status, payload) = match result {
            Ok(v) => (200, v),
            Err(e) => (400, Value::String(e)),
        };
        let bytes = serde_json::to_vec(&payload).unwrap_or_default();
        return reply(request, status, "application/json", bytes, None);
    }
    if request.method() != &Method::Get {
        return reply(
            request,
            405,
            "text/plain",
            b"Method not allowed".to_vec(),
            None,
        );
    }
    if url == "/api/ping" {
        return reply(
            request,
            200,
            "application/json",
            b"\"selah\"".to_vec(),
            None,
        );
    }
    let path = match url.trim_start_matches('/') {
        "" => "index.html",
        p => p,
    };
    match app.asset_resolver().get(path.to_string()) {
        Some(asset) => {
            let csp = asset.csp_header.clone();
            reply(request, 200, &asset.mime_type, asset.bytes, csp)
        }
        None => reply(request, 404, "text/plain", b"Not found".to_vec(), None),
    }
}

pub fn start(app: tauri::AppHandle) {
    let server = match Server::http(("0.0.0.0", PORT)) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("Network access unavailable on port {PORT}: {e}");
            return;
        }
    };
    println!(
        "Selah is available on the network at {}",
        urls().join(" or ")
    );
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let app = app.clone();
            std::thread::spawn(move || handle(&app, request));
        }
    });
}
