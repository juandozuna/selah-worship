use serde::{Deserialize, Serialize};

pub const TRANSITION_KINDS: [&str; 9] = [
    "cut", "fade", "dip", "slide", "push", "wipe", "reveal", "zoom", "blur",
];
pub const TEXT_KINDS: [&str; 7] = [
    "none",
    "fade",
    "slide",
    "blur",
    "typewriter",
    "word-rise",
    "line-cascade",
];
const EASINGS: [&str; 5] = ["linear", "ease-in", "ease-out", "ease-in-out", "spring"];
const DIRECTIONS: [&str; 4] = ["left", "right", "up", "down"];
const UNITS: [&str; 4] = ["block", "line", "word", "letter"];
const ORDERS: [&str; 4] = ["forward", "backward", "center", "random"];

/// Block-to-block transition. `direction` is the way content moves on screen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransitionSpec {
    pub kind: String,
    pub duration_ms: u32,
    pub easing: String,
    pub direction: String,
    pub feather: f32,
}
impl Default for TransitionSpec {
    fn default() -> Self {
        Self {
            kind: "cut".into(),
            duration_ms: 400,
            easing: "ease-in-out".into(),
            direction: "left".into(),
            feather: 0.1,
        }
    }
}
impl TransitionSpec {
    pub fn validate(&self) -> Result<(), String> {
        if !TRANSITION_KINDS.contains(&self.kind.as_str())
            || !EASINGS.contains(&self.easing.as_str())
            || !DIRECTIONS.contains(&self.direction.as_str())
            || !(100..=3000).contains(&self.duration_ms)
            || !(0.0..=0.5).contains(&self.feather)
        {
            return Err("Invalid transition settings".into());
        }
        Ok(())
    }
}

/// Text animation applied to each text region when a block appears.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextAnimSpec {
    pub kind: String,
    pub unit: String,
    pub duration_ms: u32,
    pub stagger_ms: u32,
    pub easing: String,
    pub direction: String,
    pub order: String,
}
impl Default for TextAnimSpec {
    fn default() -> Self {
        Self {
            kind: "none".into(),
            unit: "line".into(),
            duration_ms: 500,
            stagger_ms: 80,
            easing: "ease-out".into(),
            direction: "up".into(),
            order: "forward".into(),
        }
    }
}
impl TextAnimSpec {
    pub fn validate(&self) -> Result<(), String> {
        if !TEXT_KINDS.contains(&self.kind.as_str())
            || !UNITS.contains(&self.unit.as_str())
            || !EASINGS.contains(&self.easing.as_str())
            || !DIRECTIONS.contains(&self.direction.as_str())
            || !ORDERS.contains(&self.order.as_str())
            || !(100..=3000).contains(&self.duration_ms)
            || self.stagger_ms > 1000
        {
            return Err("Invalid text animation settings".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotionSettings {
    #[serde(default)]
    pub transition: TransitionSpec,
    #[serde(default)]
    pub text: TextAnimSpec,
}
impl MotionSettings {
    pub fn validate(&self) -> Result<(), String> {
        self.transition.validate()?;
        self.text.validate()
    }
}

/// Per-song or per-block override; missing parts inherit from the next level up.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotionOverride {
    #[serde(default)]
    pub transition: Option<TransitionSpec>,
    #[serde(default)]
    pub text: Option<TextAnimSpec>,
}
impl MotionOverride {
    pub fn validate(&self) -> Result<(), String> {
        self.transition.as_ref().map_or(Ok(()), |t| t.validate())?;
        self.text.as_ref().map_or(Ok(()), |t| t.validate())
    }
}
/// Block over song over project default.
pub fn resolve_motion(
    base: &MotionSettings,
    song: Option<&MotionOverride>,
    block: Option<&MotionOverride>,
) -> MotionSettings {
    let mut out = base.clone();
    for o in [song, block].into_iter().flatten() {
        if let Some(t) = &o.transition {
            out.transition = t.clone();
        }
        if let Some(t) = &o.text {
            out.text = t.clone();
        }
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PresentationBlock {
    pub id: String,
    pub section: String,
    pub primary: String,
    pub secondary: Option<String>,
    pub reference: Option<String>,
    #[serde(default)]
    pub motion: Option<MotionOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub name: String,
    pub url: Option<String>,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub language: String,
    pub kind: String,
    pub source: Source,
    pub blocks: Vec<PresentationBlock>,
    #[serde(default)]
    pub motion: Option<MotionOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextRegion {
    pub id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub font_family: String,
    pub color: String,
    pub align: String,
    pub binding: String,
    pub outline: f32,
    pub shadow: bool,
    /// Text animation preset for this region: `None` inherits the resolved default.
    #[serde(default)]
    pub anim: Option<String>,
}

/// Image or video file stored under the app media library.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: String,
    pub name: String,
    /// Relative filename under the media directory, or an absolute path.
    pub path: String,
    /// `image` or `video`.
    pub kind: String,
    pub mime: String,
}

impl MediaAsset {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.name.is_empty() || self.path.is_empty() {
            return Err("Media assets need an id, name, and path".into());
        }
        if !["image", "video"].contains(&self.kind.as_str()) {
            return Err("Media kind must be image or video".into());
        }
        if self.mime.is_empty() || self.mime.len() > 200 {
            return Err("Invalid media MIME type".into());
        }
        Ok(())
    }
}

/// A background (behind lyrics) or foreground (in front of lyrics) media rect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaLayer {
    pub id: String,
    pub name: String,
    /// `background` or `foreground`.
    pub role: String,
    /// `image` or `video`.
    pub kind: String,
    pub asset_id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub opacity: f32,
    pub muted: bool,
    #[serde(default)]
    pub audio_device_id: Option<String>,
    /// `cover`, `contain`, or `fill`.
    #[serde(default = "default_fit")]
    pub fit: String,
}

fn default_fit() -> String {
    "cover".into()
}

impl MediaLayer {
    pub fn validate(&self, canvas_w: u32, canvas_h: u32) -> Result<(), String> {
        if self.id.is_empty() || self.name.is_empty() || self.asset_id.is_empty() {
            return Err("Media layers need an id, name, and asset".into());
        }
        if !["background", "foreground"].contains(&self.role.as_str()) {
            return Err("Media role must be background or foreground".into());
        }
        if !["image", "video"].contains(&self.kind.as_str()) {
            return Err("Media kind must be image or video".into());
        }
        if !["cover", "contain", "fill"].contains(&self.fit.as_str()) {
            return Err("Media fit must be cover, contain, or fill".into());
        }
        if [self.x, self.y, self.width, self.height, self.opacity]
            .iter()
            .any(|v| !v.is_finite())
            || self.x < 0.0
            || self.y < 0.0
            || self.width <= 0.0
            || self.height <= 0.0
            || self.x + self.width > canvas_w as f32 + 0.01
            || self.y + self.height > canvas_h as f32 + 0.01
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err(format!(
                "Media layer '{}' is outside the canvas or has invalid opacity",
                self.name
            ));
        }
        if let Some(dev) = &self.audio_device_id {
            if dev.len() > 500 {
                return Err("Audio device id is too long".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasLayout {
    pub width: u32,
    pub height: u32,
    pub regions: Vec<TextRegion>,
    #[serde(default)]
    pub media_layers: Vec<MediaLayer>,
}

impl CanvasLayout {
    pub fn validate(&self) -> Result<(), String> {
        if self.width < 64
            || self.height < 64
            || self.width > 8192
            || self.height > 8192
            || self.width as u64 * self.height as u64 > 33_554_432
        {
            return Err(
                "Canvas must be between 64 and 8192 pixels per side, at most 32 megapixels".into(),
            );
        }
        if self.regions.is_empty() || self.regions.len() > 32 {
            return Err("Use 1–32 text regions".into());
        }
        if self.media_layers.len() > 8 {
            return Err("Use at most 8 media layers".into());
        }
        let mut ids = std::collections::HashSet::new();
        for r in &self.regions {
            if r.id.is_empty() || !ids.insert(&r.id) {
                return Err("Text regions need unique IDs".into());
            }
            if [r.x, r.y, r.width, r.height, r.font_size, r.outline]
                .iter()
                .any(|v| !v.is_finite())
                || r.x < 0.0
                || r.y < 0.0
                || r.width <= 0.0
                || r.height <= 0.0
                || r.x + r.width > self.width as f32 + 0.01
                || r.y + r.height > self.height as f32 + 0.01
                || !(8.0..=512.0).contains(&r.font_size)
                || !(0.0..=20.0).contains(&r.outline)
            {
                return Err(format!(
                    "Region '{}' is outside the canvas or has invalid dimensions",
                    r.name
                ));
            }
            if r.anim
                .as_ref()
                .is_some_and(|a| !TEXT_KINDS.contains(&a.as_str()))
            {
                return Err("Invalid region animation".into());
            }
            if !["primary", "secondary", "reference"].contains(&r.binding.as_str())
                || !["left", "center", "right"].contains(&r.align.as_str())
                || r.color.len() != 7
                || !r.color.starts_with('#')
                || !r.color[1..].chars().all(|c| c.is_ascii_hexdigit())
            {
                return Err("Invalid text style or binding".into());
            }
        }
        let mut media_ids = std::collections::HashSet::new();
        let mut bg = 0usize;
        let mut fg = 0usize;
        for layer in &self.media_layers {
            layer.validate(self.width, self.height)?;
            if !media_ids.insert(&layer.id) {
                return Err("Media layers need unique IDs".into());
            }
            match layer.role.as_str() {
                "background" => bg += 1,
                "foreground" => fg += 1,
                _ => {}
            }
        }
        if bg > 2 || fg > 2 {
            return Err("Use at most two background and two foreground media layers".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub language: String,
    pub feed_name: String,
    pub ai_provider: String,
    pub ai_model: String,
    pub ai_endpoint: String,
    pub local_server_path: String,
    pub local_model_path: String,
}

/// A physical full-screen output with its own layout and background color.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOutput {
    pub monitor: usize,
    pub background: String,
    pub layout: CanvasLayout,
}

impl DisplayOutput {
    pub fn validate(&self) -> Result<(), String> {
        if self.background.len() != 7
            || !self.background.starts_with('#')
            || !self.background[1..].chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err("Choose a valid display background color".into());
        }
        self.layout.validate()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub version: u32,
    pub name: String,
    pub library: Vec<Song>,
    pub service: Vec<String>,
    pub layout: CanvasLayout,
    pub settings: Settings,
    #[serde(default)]
    pub motion: MotionSettings,
    #[serde(default)]
    pub display: Option<DisplayOutput>,
    /// Media files referenced by layout media layers.
    #[serde(default)]
    pub media: Vec<MediaAsset>,
    /// When true, NDI/Syphon publish the full media+lyrics composite; otherwise lyrics-only alpha.
    #[serde(default)]
    pub stream_composite: bool,
}

impl Project {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported service file version".into());
        }
        self.layout.validate()?;
        self.motion.validate()?;
        if let Some(display) = &self.display {
            display.validate()?;
        }
        if self.media.len() > 64 {
            return Err("Media library is too large".into());
        }
        let mut media_ids = std::collections::HashSet::new();
        for asset in &self.media {
            asset.validate()?;
            if !media_ids.insert(&asset.id) {
                return Err("Media assets need unique IDs".into());
            }
        }
        for layer in self
            .layout
            .media_layers
            .iter()
            .chain(
                self.display
                    .as_ref()
                    .into_iter()
                    .flat_map(|d| d.layout.media_layers.iter()),
            )
        {
            if !media_ids.contains(&layer.asset_id) {
                return Err(format!(
                    "Media layer '{}' references a missing asset",
                    layer.name
                ));
            }
            if let Some(asset) = self.media.iter().find(|a| a.id == layer.asset_id) {
                if asset.kind != layer.kind {
                    return Err(format!(
                        "Media layer '{}' kind does not match its asset",
                        layer.name
                    ));
                }
            }
        }
        if self.library.len() > 10_000 {
            return Err("Library is too large".into());
        }
        let mut ids = std::collections::HashSet::new();
        for song in &self.library {
            if song.id.is_empty()
                || !ids.insert(&song.id)
                || song.blocks.is_empty()
                || song.blocks.len() > 10_000
            {
                return Err("Content must have a unique ID and at least one block".into());
            }
            if let Some(m) = &song.motion {
                m.validate()?;
            }
            let mut block_ids = std::collections::HashSet::new();
            if song.blocks.iter().any(|b| {
                b.id.is_empty()
                    || !block_ids.insert(&b.id)
                    || b.primary.len() > 100_000
                    || b.motion.as_ref().is_some_and(|m| m.validate().is_err())
            }) {
                return Err("Invalid content block".into());
            }
        }
        if self.service.iter().any(|id| !ids.contains(id)) {
            return Err("Service references missing content".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiveSnapshot {
    pub song_id: Option<String>,
    pub block_index: Option<usize>,
    pub block: Option<PresentationBlock>,
    pub layout: CanvasLayout,
    pub test_pattern: bool,
    /// The display applies its own bilingual layout while live AI translation is enabled.
    #[serde(default)]
    pub ai_translation: bool,
    /// Resolved transition and text animation for this change; `None` is a hard cut.
    #[serde(default)]
    pub motion: Option<MotionSettings>,
    /// Assets needed to resolve layout media layers for display and optional stream composite.
    #[serde(default)]
    pub media: Vec<MediaAsset>,
    /// When true, NDI/Syphon bake media under/over lyrics; physical display always composites.
    #[serde(default)]
    pub stream_composite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleVerse {
    pub translation: String,
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

/// Model output refers to source lines; it never supplies replacement words.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisGroup {
    pub section: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchCandidate {
    pub title: String,
    pub url: String,
    pub description: String,
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    #[test]
    fn block_overrides_song_overrides_project() {
        let base = MotionSettings::default();
        let fade = TransitionSpec {
            kind: "fade".into(),
            ..Default::default()
        };
        let wipe = TransitionSpec {
            kind: "wipe".into(),
            ..Default::default()
        };
        let song = MotionOverride {
            transition: Some(fade.clone()),
            text: None,
        };
        let block = MotionOverride {
            transition: Some(wipe.clone()),
            text: None,
        };
        assert_eq!(resolve_motion(&base, None, None), base);
        assert_eq!(resolve_motion(&base, Some(&song), None).transition, fade);
        assert_eq!(
            resolve_motion(&base, Some(&song), Some(&block)).transition,
            wipe
        );
        assert_eq!(
            resolve_motion(&base, Some(&song), Some(&block)).text,
            base.text
        );
    }
    #[test]
    fn rejects_unknown_kinds_and_out_of_range_durations() {
        assert!(TransitionSpec {
            kind: "spin".into(),
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(TransitionSpec {
            duration_ms: 5,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(TextAnimSpec {
            stagger_ms: 5000,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(MotionSettings::default().validate().is_ok());
    }
    #[test]
    fn old_projects_without_motion_fields_still_parse() {
        let m: MotionSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(m.transition.kind, "cut");
    }
    #[test]
    fn old_layouts_without_media_still_parse() {
        let layout: CanvasLayout = serde_json::from_str(
            "{\"width\":1920,\"height\":1080,\"regions\":[{\"id\":\"primary\",\"name\":\"Lyrics\",\"x\":0,\"y\":0,\"width\":100,\"height\":100,\"fontSize\":40,\"fontFamily\":\"Arial\",\"color\":\"#ffffff\",\"align\":\"center\",\"binding\":\"primary\",\"outline\":0,\"shadow\":true}]}",
        )
        .unwrap();
        assert!(layout.media_layers.is_empty());
        assert!(layout.validate().is_ok());
    }
    #[test]
    fn media_layer_validation_rejects_bad_geometry_and_roles() {
        let layer = MediaLayer {
            id: "bg".into(),
            name: "BG".into(),
            role: "background".into(),
            kind: "image".into(),
            asset_id: "a1".into(),
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            opacity: 1.0,
            muted: true,
            audio_device_id: None,
            fit: "cover".into(),
        };
        assert!(layer.validate(1920, 1080).is_ok());
        let mut bad = layer.clone();
        bad.role = "mid".into();
        assert!(bad.validate(1920, 1080).is_err());
        bad = layer.clone();
        bad.opacity = 2.0;
        assert!(bad.validate(1920, 1080).is_err());
    }
}
