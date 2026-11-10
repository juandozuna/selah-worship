# Presentation command bus, Wave 1

The shared Rust/TypeScript command is an internally tagged camelCase object:
`{type:"go",songId:"song-id",blockIndex:0}`. Other types: `next`, `prev`,
`clear`, `chaseArm` (`source`: `none`/`mtc`/`ltc`), `chaseResync`,
`timelinePlay`, `timelinePause`, `timelineSeek` (`timeFrames`).

`selah_application::presenter::apply_presentation_command` is a pure applicator.
It validates the project and command and returns `{live: LiveSnapshot|null,status}`.
It reuses `present`/`clear_with_media`, resolving project → song → block motion.
The current frontend queue and `set_live` output engine remain the publication path.
Wave 1 does not migrate keyboard handling or start adapters.

## Tauri and LAN API

Registered in both the invoke handler and existing HTTP JSON dispatcher:

- `apply_presentation_command({project,current,status,command,origin,selectedSongId,motionOn})`
  returns a `CommandIntent`. `selectedSongId` is nullable/optional; `origin` is
  `keyboard`/`midi`/`osc`/`timeline`; `motionOn` is required.
- `control_status()` reports capabilities: `commandApiVersion:1`,
  `midiListening:false`, `oscListening:false`, `mtcAvailable:false`,
  `ltcAvailable:false`, `timelineClockRunning:false`. It does not claim to report
  active runtime state or probe hardware.

The caller owns `ControlStatus`, initialized with `armedSource` from project
`settings.timecode`, `chasePaused:false`, `timelinePlaying:false`, `timeFrames:0`.
Serialize **preparation and publication together** in one queue. Read the latest
accepted live/status inside that queue, apply the command, await the existing
`bridge.present(intent.live)` when non-null, then accept the returned status/live.
Do not feed several independently prepared snapshots into competing output paths.
Failed output publication must not advance accepted live/status.

Next/Prev stay within the current song with no wrapping. When clear, they start
block zero of the selected song, falling back to the first service song. Boundaries
produce no live intent. Manual Next/Prev/Clear from any non-timeline origin pauses
chase and playback when `pauseOnManual` is enabled. Re-sync clears the chase pause.
Arming replaces the exclusive source; `none` disarms. Play/pause/seek only change
transport state; Wave 2 supplies the clock and cue scheduling.

## Persisted contracts and Wave 2 hooks

Missing new fields deserialize with defaults: MIDI disabled, channel 1 Note
60/61/62 → Next/Prev/Clear, OSC disabled on UDP 9000, timecode 30 NDF with chase
`none` and pause-on-manual enabled; songs have no `cues`, projects no `fonts`.
TS fields are optional for legacy objects; exported defaults live in
`src/controlModel.ts`. Explicit malformed/null configuration is rejected.

Cues live on `Song.cues` with project-wide unique IDs, absolute elapsed
`timeFrames` (safe unsigned JS integers), and a full command. At the same frame,
Wave 2 should preserve array order. FPS accepts 24/25/29.97/30; drop-frame is
valid only at 29.97. DF affects label conversion, not elapsed frame counts.
Font assets live on `Project.fonts` with unique IDs, family, and relative
`fonts/*.ttf` or `fonts/*.otf` sidecar paths. Validation checks portability;
filesystem availability and missing-font warnings belong to import/export in Wave 2.
System font selection continues to use `TextRegion.fontFamily`.

Wave 2: route keyboard, MIDI learn/edit map, OSC and timeline events through one
queue in Keyboard → MIDI → OSC → Timeline priority order. MIDI note bindings
fire only on positive note-on; control-change value filtering/learn belongs to
the adapter. OSC routes are `/selah/next`, `/selah/prev`, `/selah/clear`,
`/selah/go/{songId}/{block}` (zero-based block), `/selah/chase/arm` (source argument).
Attach MTC/LTC decoding, exclusive chase gating and re-sync, frame conversion,
cue scheduling, and scrub/play/tap-set-cue UI. No listeners, decoders, clock,
font importing, renderer changes, or NDI changes ship in Wave 1.
