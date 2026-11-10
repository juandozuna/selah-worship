//! Persisted adapter configuration and the shared, transport-independent command vocabulary.
use crate::Project;
use serde::{Deserialize, Serialize};

pub const MAX_TIME_FRAMES: u64 = 9_007_199_254_740_991; // JS safe integer
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChaseSource {
    #[default]
    None,
    Mtc,
    Ltc,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PresentationCommand {
    Next,
    Prev,
    Clear,
    Go { song_id: String, block_index: usize },
    ChaseArm { source: ChaseSource },
    ChaseResync,
    TimelinePlay,
    TimelinePause,
    TimelineSeek { time_frames: u64 },
}
impl PresentationCommand {
    pub fn validate(&self, project: &Project) -> Result<(), String> {
        match self {
            Self::Go {
                song_id,
                block_index,
            } => {
                if !project
                    .library
                    .iter()
                    .any(|s| s.id == *song_id && *block_index < s.blocks.len())
                {
                    return Err("Command references missing content or block".into());
                }
            }
            Self::TimelineSeek { time_frames } if *time_frames > MAX_TIME_FRAMES => {
                return Err("Timeline position exceeds safe frame range".into());
            }
            _ => {}
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandOrigin {
    Keyboard,
    Midi,
    Osc,
    Timeline,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MidiMessage {
    Note,
    ControlChange,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MidiBinding {
    pub id: String,
    /// One-based MIDI channel (1–16).
    pub channel: u8,
    pub message: MidiMessage,
    /// Note or controller number (0–127). Note bindings fire on positive note-on only.
    pub number: u8,
    pub command: PresentationCommand,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MidiMap {
    pub enabled: bool,
    pub device_id: Option<String>,
    pub bindings: Vec<MidiBinding>,
}
impl Default for MidiMap {
    fn default() -> Self {
        Self {
            enabled: false,
            device_id: None,
            bindings: [
                PresentationCommand::Next,
                PresentationCommand::Prev,
                PresentationCommand::Clear,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, command)| MidiBinding {
                id: format!("default-{i}"),
                channel: 1,
                message: MidiMessage::Note,
                number: 60 + i as u8,
                command,
            })
            .collect(),
        }
    }
}
impl MidiMap {
    pub fn validate(&self, project: &Project) -> Result<(), String> {
        if self.bindings.len() > 1024
            || self
                .device_id
                .as_ref()
                .is_some_and(|d| d.is_empty() || d.len() > 500)
        {
            return Err("Invalid MIDI map".into());
        }
        let mut ids = std::collections::HashSet::new();
        let mut inputs = std::collections::HashSet::new();
        for b in &self.bindings {
            if b.id.is_empty()
                || !ids.insert(&b.id)
                || !(1..=16).contains(&b.channel)
                || b.number > 127
                || !inputs.insert((b.channel, b.message as u8, b.number))
            {
                return Err("Invalid or duplicate MIDI binding".into());
            }
            b.command.validate(project)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OscSettings {
    pub enabled: bool,
    pub port: u16,
}
impl Default for OscSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 9000,
        }
    }
}
impl OscSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("OSC port must be 1–65535".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimecodeSettings {
    pub fps: f64,
    pub drop_frame: bool,
    /// Exclusive source selection; `none` disarms chase.
    pub armed_source: ChaseSource,
    pub pause_on_manual: bool,
}
impl Default for TimecodeSettings {
    fn default() -> Self {
        Self {
            fps: 30.0,
            drop_frame: false,
            armed_source: ChaseSource::None,
            pause_on_manual: true,
        }
    }
}
impl TimecodeSettings {
    pub fn validate(&self) -> Result<(), String> {
        if ![24.0, 25.0, 29.97, 30.0].contains(&self.fps) || (self.drop_frame && self.fps != 29.97)
        {
            return Err("Invalid timecode FPS/drop-frame combination".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineCue {
    pub id: String,
    /// Absolute elapsed frame count in the project's timebase (not an encoded DF label).
    pub time_frames: u64,
    pub command: PresentationCommand,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FontAsset {
    pub id: String,
    pub family: String,
    pub relative_path: String,
}
impl FontAsset {
    pub fn validate(&self) -> Result<(), String> {
        let p = &self.relative_path;
        if self.id.trim().is_empty()
            || self.family.trim().is_empty()
            || !p.starts_with("fonts/")
            || p.contains(['\\', ':', '\0'])
            || p.split('/').any(|s| s.is_empty() || s == "." || s == "..")
            || !["ttf", "otf"]
                .iter()
                .any(|ext| p.to_lowercase().ends_with(&format!(".{ext}")))
        {
            return Err(
                "Font assets require a family and a relative fonts/*.ttf or *.otf path".into(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn legacy() -> Project {
        serde_json::from_str(include_str!("../../../tests/fixtures/control-project.json")).unwrap()
    }
    #[test]
    fn legacy_defaults_and_wire_format() {
        let p = legacy();
        p.validate().unwrap();
        assert_eq!(
            p.settings
                .midi
                .bindings
                .iter()
                .map(|b| b.number)
                .collect::<Vec<_>>(),
            vec![60, 61, 62]
        );
        assert!(p.fonts.is_empty() && p.library[0].cues.is_empty());
        assert_eq!(p.settings.timecode, TimecodeSettings::default());
        let command = PresentationCommand::Go {
            song_id: "song".into(),
            block_index: 1,
        };
        assert_eq!(
            serde_json::to_value(&command).unwrap(),
            serde_json::json!({"type":"go","songId":"song","blockIndex":1})
        );
        let roundtrip: Project = serde_json::from_value(serde_json::to_value(p).unwrap()).unwrap();
        roundtrip.validate().unwrap();
    }
    #[test]
    fn rejects_invalid_controls_and_targets() {
        let mut p = legacy();
        p.settings.osc.port = 0;
        assert!(p.validate().is_err());
        p.settings.osc = OscSettings::default();
        p.settings.timecode.drop_frame = true;
        assert!(p.validate().is_err());
        p.settings.timecode.fps = 29.97;
        p.validate().unwrap();
        p.settings.midi.bindings[0].channel = 0;
        assert!(p.validate().is_err());
        p.settings.midi = MidiMap::default();
        p.settings
            .midi
            .bindings
            .push(p.settings.midi.bindings[0].clone());
        assert!(p.validate().is_err());
        p.settings.midi = MidiMap::default();
        p.library[0].cues.push(TimelineCue {
            id: "cue".into(),
            time_frames: 0,
            command: PresentationCommand::Go {
                song_id: "song".into(),
                block_index: 2,
            },
        });
        assert!(p.validate().is_err());
        p.library[0].cues[0].command = PresentationCommand::Clear;
        p.validate().unwrap();
        p.library[0].cues[0].time_frames = MAX_TIME_FRAMES + 1;
        assert!(p.validate().is_err());
    }
    #[test]
    fn font_paths_are_portable() {
        for path in [
            "/tmp/font.ttf",
            "fonts/../font.ttf",
            "fonts/a\\b.otf",
            "fonts/C:font.ttf",
            "fonts/font.woff",
            "fonts//font.ttf",
        ] {
            assert!(
                FontAsset {
                    id: "f".into(),
                    family: "Family".into(),
                    relative_path: path.into()
                }
                .validate()
                .is_err(),
                "{path}"
            );
        }
        let mut p = legacy();
        p.fonts.push(FontAsset {
            id: "f".into(),
            family: "Family".into(),
            relative_path: "fonts/family.otf".into(),
        });
        p.validate().unwrap();
        p.fonts.push(p.fonts[0].clone());
        assert!(p.validate().is_err());
    }
}
