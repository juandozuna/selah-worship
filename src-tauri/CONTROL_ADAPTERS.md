# Native control adapters (Wave 2)

Native adapters prepare commands only. Subscribe once in the desktop controller to
`control://command` with Tauri's `listen`, before loading/starting the project:

```ts
const unlisten = await listen<{ command: PresentationCommand; origin: CommandOrigin }>(
  'control://command', ({ payload }) => enqueuePresentationCommand(payload)
);
```

The existing React priority queue must apply and publish each event. **After each
successful queued publication**, invoke `control_sync({status: intent.status})`.
Also synchronize accepted commands with no live intent (arm, pause, seek, resync,
play). Do not sync failed publication or speculative preparation. This lets the
adapter gate clocks using the applicator's accepted exclusive source and pause
state, without reimplementing manual pause rules. Timeline frame updates arrive
as `timelineSeek` commands, followed by crossed cues, all with origin `timeline`.
The frontend remains the sole output publisher. No frontend files changed here.
Native events are local to Tauri; the LAN HTTP dispatcher supports the invokes
below but does not relay events to browser clients.

## Invokes

- `control_start({project})`: validate project, restart listeners, initialize
  transport with the project's armed source. `load_project` does this automatically.
- `save_project({project})`: update cues; restart adapters only if adapter settings
  changed, preserving accepted pause/play state. Source changes in persisted
  timecode settings replace the armed source.
- `control_stop()`: close MIDI/audio/socket listeners and join their owner thread.
  App shutdown also stops them. Startup is asynchronous: query `control_status`
  for actual listening flags and device/bind errors. An unavailable device does
  not prevent loading a project or starting other adapters. Retry via control_start.
- `control_sync({status})`: mirror accepted `ControlStatus`. Changing source restarts
  the relevant input; only one of MTC/LTC can drive chase. The internal project-FPS
  clock runs only for timeline play with no external source armed.
- `list_midi_devices()`: `{id,name}[]`; IDs are native port names. Null deviceId uses
  the first available input. Duplicate native names cannot be distinguished.
- `midi_learn({bindingId})`: one-shot probe, including when mappings are disabled.
  Subscribe to `control://midi-learn`, payload `{bindingId,channel,message,number}`.
  Channel is 1–16, message is `note` or `controlChange`. Capture suppresses the
  mapped command for that message. Pass `bindingId:null` to cancel. A project must
  already be loaded. Positive note-on and every CC value (including zero) qualify;
  note-off and zero-velocity note-on do not.
- `list_ltc_audio_devices()`: audio **input** devices as `{id,label}[]`. Current
  persisted settings have no LTC device/channel selector, so acquisition uses
  the OS default input's first channel, supporting f32/i16/u16 samples.
- `import_font({path?})`: desktop-only; omit path to open a native font picker.
  Validate .ttf/.otf font data (32 MiB limit), copy to app media `fonts/`, return
  `{id,family,relativePath}`. Add the asset to Project.fonts and save it. IDs and
  filenames use a SHA-256 content digest, so importing the same file is stable.
  Fonts load into the shared renderer fontdb on import and project load/save,
  including existing preview/output renderers. Browser/CSS FontFace registration
  remains the frontend's responsibility. Missing sidecars use system fallback.

## Protocol details

OSC binds IPv4 UDP on the configured port when enabled. Routes:
`/selah/next`, `/selah/prev`, `/selah/clear`, `/selah/go/{songId}/{blockIndex}`
(no arguments), and `/selah/chase/arm` with exactly one string `none`, `mtc`, or
`ltc`. Invalid routes/arguments are ignored. Immediate bundles retain wire order;
non-immediate bundles are ignored rather than executed before their timetag.

MTC accepts forward ordered quarter-frame cycles and full-frame SysEx locates.
Quarter-frame completion includes the standard two-frame forward compensation;
full-frame locates do not. Incoming rate must match the project: 24, 25, 29.97 DF,
or 30; project 29.97 NDF accepts the 30/NDF MTC rate code. Reverse MTC/LTC playback
is not supported. LTC decoding uses the pure Rust `ltc` decoder over cpal input;
a bounded channel keeps frame scheduling/event emission off the audio callback.
Audio input may require OS microphone permission.

Both sources convert SMPTE labels to absolute elapsed frame counts before cue
comparison. DF removes skipped labels only. On initial lock/resync, only cues
exactly at that position fire. Subsequent forward movement fires crossed cues
chronologically, preserving library/song cue array order for ties. Duplicate
frames do not retrigger. A rewind rebases without firing cues backwards; future
forward crossings can retrigger. Paused chase consumes positions without firing.
Clock running reflects a signal within the last second and accepted pause state.

## Validation

`cargo check --manifest-path src-tauri/Cargo.toml`

`cargo test --manifest-path src-tauri/Cargo.toml`

`cargo test -p selah-domain -p selah-application`

Unit coverage includes MIDI filtering, actual UDP reception, strict OSC parsing,
MTC assembly/rates, DF conversion, synthetic LTC audio decoding, exclusive source
gating, delayed clock acknowledgements, crossed cues, ties, pause and rewind.
Physical MIDI/audio devices and OS permission prompts require hardware verification.
