//! Pure preparation stage of the command bus. The caller serializes this with its
//! existing present/set_live queue and publishes only `live: Some` intents.
use selah_domain::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ControlStatus {
    pub armed_source: ChaseSource,
    pub chase_paused: bool,
    pub timeline_playing: bool,
    pub time_frames: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandIntent {
    pub live: Option<LiveSnapshot>,
    pub status: ControlStatus,
}
/// Within-song stepping matches App.tsx: no wrapping or automatic song advance.
/// With clear output, start at block zero of selected song (or first service song).
/// `origin` is transport metadata, never inferred from the command payload.
pub fn apply_presentation_command(
    project: &Project,
    current: &LiveSnapshot,
    status: &ControlStatus,
    command: &PresentationCommand,
    origin: CommandOrigin,
    selected_song_id: Option<&str>,
    motion_on: bool,
) -> Result<CommandIntent, String> {
    project.validate()?;
    command.validate(project)?;
    if status.time_frames > MAX_TIME_FRAMES {
        return Err("Invalid control position".into());
    }
    let mut next = status.clone();
    if origin != CommandOrigin::Timeline
        && project.settings.timecode.pause_on_manual
        && matches!(
            command,
            PresentationCommand::Next | PresentationCommand::Prev | PresentationCommand::Clear
        )
    {
        next.chase_paused = true;
        next.timeline_playing = false;
    }
    let target = match command {
        PresentationCommand::Go {
            song_id,
            block_index,
        } => Some((song_id.as_str(), *block_index)),
        PresentationCommand::Next | PresentationCommand::Prev => {
            let song = current
                .song_id
                .as_deref()
                .and_then(|id| project.library.iter().find(|s| s.id == id))
                .or_else(|| {
                    selected_song_id.and_then(|id| project.library.iter().find(|s| s.id == id))
                })
                .or_else(|| {
                    project
                        .service
                        .first()
                        .and_then(|id| project.library.iter().find(|s| s.id == *id))
                });
            song.and_then(|s| {
                let index = match current
                    .block_index
                    .filter(|_| current.song_id.as_deref() == Some(s.id.as_str()))
                {
                    None => Some(0),
                    Some(i) if matches!(command, PresentationCommand::Next) => i.checked_add(1),
                    Some(i) => i.checked_sub(1),
                };
                index
                    .filter(|i| *i < s.blocks.len())
                    .map(|i| (s.id.as_str(), i))
            })
        }
        PresentationCommand::ChaseArm { source } => {
            next.armed_source = *source;
            None
        }
        PresentationCommand::ChaseResync => {
            next.chase_paused = false;
            None
        }
        PresentationCommand::TimelinePlay => {
            next.timeline_playing = true;
            None
        }
        PresentationCommand::TimelinePause => {
            next.timeline_playing = false;
            None
        }
        PresentationCommand::TimelineSeek { time_frames } => {
            next.time_frames = *time_frames;
            None
        }
        PresentationCommand::Clear => None,
    };
    let live = if matches!(command, PresentationCommand::Clear) {
        let mut live = crate::clear_with_media(project);
        live.motion = motion_on.then(|| project.motion.clone());
        Some(live)
    } else if let Some((id, index)) = target {
        let mut live = crate::present(project, id, index)?;
        let song = project.library.iter().find(|s| s.id == id).unwrap();
        live.motion = motion_on.then(|| {
            resolve_motion(
                &project.motion,
                song.motion.as_ref(),
                song.blocks[index].motion.as_ref(),
            )
        });
        Some(live)
    } else {
        None
    };
    Ok(CommandIntent { live, status: next })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project() -> Project {
        serde_json::from_str(include_str!("../../../tests/fixtures/control-project.json")).unwrap()
    }
    #[test]
    fn navigation_boundaries_and_clear_preserve_media() {
        let mut p = project();
        p.media.push(MediaAsset {
            id: "bg".into(),
            name: "Background".into(),
            path: "bg.jpg".into(),
            kind: "image".into(),
            mime: "image/jpeg".into(),
        });
        p.stream_composite = true;
        p.library[0].motion = Some(MotionOverride {
            transition: Some(TransitionSpec {
                kind: "fade".into(),
                ..Default::default()
            }),
            text: None,
        });
        let clear = crate::clear_with_media(&p);
        let status = ControlStatus::default();
        let apply = |live: &LiveSnapshot, cmd: &PresentationCommand| {
            apply_presentation_command(
                &p,
                live,
                &status,
                cmd,
                CommandOrigin::Keyboard,
                Some("song"),
                true,
            )
            .unwrap()
        };
        let first = apply(&clear, &PresentationCommand::Next).live.unwrap();
        assert_eq!(first.block_index, Some(0));
        assert_eq!(first.motion.as_ref().unwrap().transition.kind, "fade");
        assert!(apply(&first, &PresentationCommand::Prev).live.is_none());
        let second = apply(&first, &PresentationCommand::Next).live.unwrap();
        assert_eq!(second.block_index, Some(1));
        assert!(apply(&second, &PresentationCommand::Next).live.is_none());
        let cleared = apply(&second, &PresentationCommand::Clear).live.unwrap();
        assert!(cleared.block.is_none());
        assert_eq!(cleared.media, p.media);
        assert!(cleared.stream_composite);
        assert!(apply_presentation_command(
            &p,
            &first,
            &status,
            &PresentationCommand::Go {
                song_id: "missing".into(),
                block_index: 0
            },
            CommandOrigin::Midi,
            None,
            true
        )
        .is_err());
    }
    #[test]
    fn manual_pause_resync_and_exclusive_chase() {
        let p = project();
        let live = crate::clear_with_media(&p);
        let apply = |s: &ControlStatus, c: &PresentationCommand, o| {
            apply_presentation_command(&p, &live, s, c, o, None, false).unwrap()
        };
        let mtc = apply(
            &ControlStatus::default(),
            &PresentationCommand::ChaseArm {
                source: ChaseSource::Mtc,
            },
            CommandOrigin::Osc,
        );
        let ltc = apply(
            &mtc.status,
            &PresentationCommand::ChaseArm {
                source: ChaseSource::Ltc,
            },
            CommandOrigin::Osc,
        );
        assert_eq!(ltc.status.armed_source, ChaseSource::Ltc);
        let paused = apply(
            &ltc.status,
            &PresentationCommand::Clear,
            CommandOrigin::Midi,
        );
        assert!(paused.status.chase_paused);
        let rearmed = apply(
            &paused.status,
            &PresentationCommand::ChaseArm {
                source: ChaseSource::Mtc,
            },
            CommandOrigin::Osc,
        );
        assert!(rearmed.status.chase_paused); // Manual pause persists until explicit re-sync.
        let resync = apply(
            &paused.status,
            &PresentationCommand::ChaseResync,
            CommandOrigin::Keyboard,
        );
        assert!(!resync.status.chase_paused);
        assert!(resync.live.is_none());
        assert!(
            !apply(
                &resync.status,
                &PresentationCommand::Next,
                CommandOrigin::Timeline
            )
            .status
            .chase_paused
        );
        let seek = apply(
            &resync.status,
            &PresentationCommand::TimelineSeek { time_frames: 99 },
            CommandOrigin::Keyboard,
        );
        assert_eq!(seek.status.time_frames, 99);
        assert!(seek.live.is_none());
        let play = apply(
            &seek.status,
            &PresentationCommand::TimelinePlay,
            CommandOrigin::Keyboard,
        );
        assert!(play.status.timeline_playing);
        assert!(
            !apply(
                &play.status,
                &PresentationCommand::TimelinePause,
                CommandOrigin::Keyboard
            )
            .status
            .timeline_playing
        );
    }
}
