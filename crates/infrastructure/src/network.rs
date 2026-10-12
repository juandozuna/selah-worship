use selah_domain::SearchCandidate;
use std::{
    io::Write,
    net::{IpAddr, ToSocketAddrs},
    process::{Command, Stdio},
};

fn escape(s: &str) -> Result<String, String> {
    if s.contains(['\n', '\r', '\0']) {
        return Err("Invalid request value".into());
    }
    Ok(s.replace('\\', "\\\\").replace('"', "\\\""))
}
/// Headers and request bodies travel over stdin, never shell arguments.
pub fn request(
    url: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
    local: bool,
) -> Result<Vec<u8>, String> {
    if !local {
        public_url(url)?;
    } else if !url.starts_with("http://127.0.0.1:47831/") {
        return Err("Invalid local AI endpoint".into());
    }
    let mut config=format!("url = \"{}\"\nmax-time = 90\nconnect-timeout = 15\nmax-filesize = 20971520\nsilent\nshow-error\nfail-with-body\n",escape(url)?);
    if !local {
        config.push_str(&format!(
            "resolve = \"{}\"\n",
            escape(&public_target(url)?)?
        ));
    }
    for (k, v) in headers {
        config.push_str(&format!("header = \"{}: {}\"\n", escape(k)?, escape(v)?));
    }
    if let Some(body) = body {
        config.push_str(&format!("data = \"{}\"\n", escape(body)?));
    }
    let child = crate::process::ManagedProcess::spawn(
        Command::new("curl")
            .args(["--config", "-", "--proto", "=http,https"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .map_err(|_| "curl is required for online requests")?;
    child
        .take_stdin()
        .ok_or("Request stdin unavailable")?
        .write_all(config.as_bytes())
        .map_err(|e| e.to_string())?;
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(format!(
            "Online request failed ({}). Check your connection, API key, and provider settings.",
            result.status
        ));
    }
    if result.stdout.len() > 20_971_520 {
        return Err("Response too large".into());
    }
    Ok(result.stdout)
}
pub fn public_url(url: &str) -> Result<(), String> {
    public_target(url).map(|_| ())
}
fn public_target(url: &str) -> Result<String, String> {
    if !url.starts_with("https://") || url.contains(['\n', '\r', '\0']) {
        return Err("Use a public HTTPS URL".into());
    }
    let host = url[8..].split('/').next().unwrap_or("");
    if host.is_empty() || host.contains(['@', ':', '[', ']']) || host == "localhost" {
        return Err("Use a public HTTPS hostname".into());
    }
    let mut first = None;
    for address in (host, 443)
        .to_socket_addrs()
        .map_err(|_| "Could not resolve hostname")?
    {
        let blocked = match address.ip() {
            IpAddr::V4(ip) => {
                ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_unspecified()
                    || ip.is_multicast()
                    || ip.is_broadcast()
            }
            IpAddr::V6(ip) => {
                ip.is_loopback()
                    || ip.is_unspecified()
                    || ip.is_multicast()
                    || ip.is_unique_local()
                    || ip.is_unicast_link_local()
            }
        };
        if first.is_none() {
            first = Some(address.ip());
        }
        if blocked {
            return Err("Private network addresses cannot be used as content sources".into());
        }
    }
    let ip = first.ok_or("No public address found")?;
    let ip = match ip {
        IpAddr::V6(ip) => format!("[{ip}]"),
        IpAddr::V4(ip) => ip.to_string(),
    };
    Ok(format!("{host}:443:{ip}"))
}

/// Resolve each redirect separately and pin the validated public address.
pub fn resolve_download(url: &str) -> Result<(String, String), String> {
    let mut current = url.to_string();
    for _ in 0..6 {
        let target = public_target(&current)?;
        let output = crate::process::ManagedProcess::spawn(
            Command::new("curl")
                .args([
                    "--head",
                    "--silent",
                    "--show-error",
                    "--max-time",
                    "20",
                    "--proto",
                    "=https",
                    "--noproxy",
                    "*",
                    "--resolve",
                    &target,
                    &current,
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        )
        .and_then(|child| child.wait_with_output())
        .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("Could not read download headers".into());
        }
        let headers = String::from_utf8_lossy(&output.stdout);
        let location = headers
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.eq_ignore_ascii_case("location"))
            .map(|(_, v)| v.trim());
        if let Some(location) = location {
            current = if location.starts_with('/') {
                format!(
                    "https://{}{}",
                    current[8..].split('/').next().unwrap_or(""),
                    location
                )
            } else {
                location.into()
            };
        } else {
            return Ok((current, target));
        }
    }
    Err("Too many download redirects".into())
}
fn encoded(s: &str) -> String {
    s.as_bytes()
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(b) {
                (*b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
pub fn search(query: &str, key: &str) -> Result<Vec<SearchCandidate>, String> {
    if query.trim().is_empty() || query.len() > 500 {
        return Err("Enter a song title or lyric phrase".into());
    }
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}&count=8",
        encoded(&format!("{query} worship song lyrics letra"))
    );
    let data = request(
        &url,
        &[
            ("X-Subscription-Token", key),
            ("Accept", "application/json"),
        ],
        None,
        false,
    )?;
    let data: serde_json::Value =
        serde_json::from_slice(&data).map_err(|_| "Invalid search response")?;
    Ok(data["web"]["results"]
        .as_array()
        .ok_or("No search results")?
        .iter()
        .filter_map(|r| {
            Some(SearchCandidate {
                title: r["title"].as_str()?.into(),
                url: r["url"].as_str()?.into(),
                description: r["description"].as_str().unwrap_or("").into(),
            })
        })
        .collect())
}
pub fn extract(url: &str) -> Result<String, String> {
    let (url, _) = resolve_download(url)?;
    let data = request(
        &url,
        &[("User-Agent", "Selah/0.1 lyric-source-review")],
        None,
        false,
    )?;
    let html = String::from_utf8(data)
        .map_err(|_| "Page is not readable UTF-8. Open it in your browser and paste the lyrics.")?;
    let html = regex::Regex::new(
        r"(?is)<(script|style|nav|header|footer)[\s>].*?</(?:script|style|nav|header|footer)>",
    )
    .unwrap()
    .replace_all(&html, "");
    let html = regex::Regex::new(r"(?i)<br\s*/?>|</(?:p|div|li|h[1-6])>")
        .unwrap()
        .replace_all(&html, "\n");
    let text = regex::Regex::new(r"(?s)<[^>]+>")
        .unwrap()
        .replace_all(&html, "");
    let text = text
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    let result = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if result.len() < 30 || result.len() > 80_000 {
        return Err(
            "Could not identify readable lyric text. Open the source page and paste the lyrics."
                .into(),
        );
    }
    Ok(result)
}
