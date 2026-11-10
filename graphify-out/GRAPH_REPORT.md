# Graph Report - LyricPassingPresentationSoftware  (2026-11-10)

## Corpus Check
- 89 files · ~195,396 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1124 nodes · 2431 edges · 65 communities (59 shown, 6 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 40 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `bef2eab3`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Canvas.tsx
- main.rs
- scripts
- LiveSnapshot
- timecode.rs
- model.ts
- bundle
- NdiPublisher
- transition.rs
- display.rs
- composite.rs
- PresentationBlock
- compilerOptions
- credentials.rs
- Selah
- definitions
- properties
- definitions
- permissions
- permissions
- ManagedProcess
- webviews
- webviews
- CapabilityRemote
- properties
- CapabilityRemote
- NSObject
- prepare-assets.mjs
- Capability
- Capability
- selah-desktop
- renderer/src/lib.rs
- react-dom.d.ts
- application/src/lib.rs
- App.tsx
- SongSearch
- MotionPicker.tsx
- check-node.mjs
- LiveTranslator
- desktop-schema.json
- control.rs
- types.ts
- local
- LayoutEditor.tsx
- remote.rs
- devDependencies
- dependencies
- listen
- package.json
- build-guide-pdf.mjs
- prepare-dmg-extras.mjs
- package-dmg.sh
- main.js
- osc.rs
- ContentDialog.tsx
- capture-screenshots.mjs
- files
- Native control adapters (Wave 2)
- macOS-schema.json
- Presentation command bus, Wave 1
- local

## God Nodes (most connected - your core abstractions)
1. `AppState` - 40 edges
2. `blocking()` - 37 edges
3. `dispatch()` - 34 edges
4. `LiveSnapshot` - 31 edges
5. `scripts` - 27 edges
6. `desktop()` - 23 edges
7. `invoke()` - 22 edges
8. `App()` - 21 edges
9. `Runtime` - 19 edges
10. `PresentationBlock` - 17 edges

## Surprising Connections (you probably didn't know these)
- `SyphonPublisher` --implements--> `OutputPublisher`  [EXTRACTED]
  src-tauri/src/output.rs → crates/application/src/lib.rs
- `raw()` --references--> `MidiMessage`  [EXTRACTED]
  src-tauri/src/midi.rs → crates/domain/src/control.rs
- `analyze_lyrics()` --references--> `PresentationBlock`  [EXTRACTED]
  src-tauri/src/main.rs → crates/domain/src/lib.rs
- `import_openlyrics()` --references--> `PresentationBlock`  [EXTRACTED]
  src-tauri/src/main.rs → crates/domain/src/lib.rs
- `DisplayFrame` --references--> `LiveSnapshot`  [EXTRACTED]
  src-tauri/src/display.rs → crates/domain/src/lib.rs

## Import Cycles
- None detected.

## Communities (65 total, 6 thin omitted)

### Community 0 - "Canvas.tsx"
Cohesion: 0.14
Nodes (26): Canvas(), clamp(), Drag, frameKey(), Guide, HANDLES, Mode, Rect (+18 more)

### Community 1 - "main.rs"
Cohesion: 0.12
Nodes (72): CommandIntent, FnOnce, MediaAsset, OutputStatus, analyze_lyrics(), apply_presentation_command(), AppState, AudioDeviceInfo (+64 more)

### Community 2 - "scripts"
Cohesion: 0.07
Nodes (27): scripts, assets, build, desktop, desktop:build, dev, dev:lan, dmg:extras (+19 more)

### Community 3 - "LiveSnapshot"
Cohesion: 0.08
Nodes (43): clear(), apply_presentation_command(), CommandIntent, ControlStatus, manual_pause_resync_and_exclusive_chase(), navigation_boundaries_and_clear_preserve_media(), project(), ChaseSource (+35 more)

### Community 4 - "timecode.rs"
Cohesion: 0.09
Nodes (36): JoinHandle, accept_status(), clock_commands(), CommandEvent, Core, cues_cross_frames_preserve_ties_and_rearm_after_rewind(), delayed_queue_ack_does_not_rewind_internal_clock(), label_frames() (+28 more)

### Community 5 - "model.ts"
Cohesion: 0.12
Nodes (25): ContentDialog(), hymn(), preset(), seed, clearLive(), CUT_MOTION, DIRECTIONS, EASINGS (+17 more)

### Community 6 - "bundle"
Cohesion: 0.05
Nodes (37): **, $APPDATA/**, $APPLOCALDATA/**, $HOME/**, icons/icon.png, app, security, windows (+29 more)

### Community 7 - "NdiPublisher"
Cohesion: 0.13
Nodes (16): c_char, Connections, OutputPublisher, Library, NdiPublisher, c_void, Drop, Option (+8 more)

### Community 8 - "transition.rs"
Cohesion: 0.19
Nodes (18): add(), box_blur(), build(), compose(), dip_passes_through_transparent_and_fade_midpoint_blends(), direction(), every_transition_starts_on_from_and_ends_on_to(), fast_blur() (+10 more)

### Community 9 - "display.rs"
Cohesion: 0.20
Nodes (18): AtomicU64, DisplayOutput, close_display(), display_frame(), DisplayFrame, DisplayInfo, DisplayOverview, DisplayState (+10 more)

### Community 10 - "composite.rs"
Cohesion: 0.08
Nodes (40): background_stays_under_lyrics_and_foreground_over(), blend_pixel(), blit_layer(), compose_stack(), dest_uv(), empty_under_keeps_lyric_alpha(), FitMode, LayerBlit (+32 more)

### Community 11 - "PresentationBlock"
Cohesion: 0.14
Nodes (29): PresentationBlock, SearchCandidate, analyze(), analyze_with_lines(), apply_local_boundaries(), extract_lyric_text_local(), local_boundary_suggestions_cannot_omit_duplicate_or_reorder_lyrics(), LocalWorker (+21 more)

### Community 12 - "compilerOptions"
Cohesion: 0.12
Nodes (16): DOM, DOM.Iterable, ES2022, src, compilerOptions, esModuleInterop, jsx, lib (+8 more)

### Community 13 - "credentials.rs"
Cohesion: 0.28
Nodes (12): CredentialStore, Credential, FileTime, get(), c_void, Result, String, Vec (+4 more)

### Community 14 - "Selah"
Cohesion: 0.09
Nodes (20): Content and intelligence, Desktop integration, Release acceptance, Reliability and accessibility, Sources and third-party components, Architecture, Automatic song search and local preparation, Desktop app with built-in network access (+12 more)

### Community 15 - "definitions"
Cohesion: 0.15
Nodes (13): definitions, Number, PermissionEntry, Target, Value, anyOf, description, anyOf (+5 more)

### Community 16 - "properties"
Cohesion: 0.15
Nodes (13): properties, Identifier, default, description, type, description, oneOf, type (+5 more)

### Community 17 - "definitions"
Cohesion: 0.15
Nodes (13): definitions, Number, PermissionEntry, Target, Value, anyOf, description, anyOf (+5 more)

### Community 18 - "permissions"
Cohesion: 0.17
Nodes (12): $ref, array, null, description, items, type, uniqueItems, description (+4 more)

### Community 19 - "permissions"
Cohesion: 0.17
Nodes (12): $ref, array, null, description, items, type, uniqueItems, description (+4 more)

### Community 20 - "ManagedProcess"
Cohesion: 0.13
Nodes (22): Child, ChildStdin, Command, dropping_owner_reaps_child(), ManagedProcess, output_is_drained_without_blocking_shutdown(), Registry, Arc (+14 more)

### Community 21 - "webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 22 - "webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 23 - "CapabilityRemote"
Cohesion: 0.22
Nodes (9): description, properties, required, type, CapabilityRemote, urls, urls, description (+1 more)

### Community 24 - "properties"
Cohesion: 0.15
Nodes (13): properties, Identifier, default, description, type, description, oneOf, type (+5 more)

### Community 25 - "CapabilityRemote"
Cohesion: 0.22
Nodes (9): description, properties, required, type, CapabilityRemote, urls, urls, description (+1 more)

### Community 26 - "NSObject"
Cohesion: 0.33
Nodes (6): NSObject, -hasClients, -initWithNamedeviceoptions, -publishFrameTextureonCommandBufferimageRegionflipped, -stop, SelahSyphonHandle

### Community 28 - "Capability"
Cohesion: 0.33
Nodes (6): description, required, type, Capability, identifier, permissions

### Community 29 - "Capability"
Cohesion: 0.33
Nodes (6): description, required, type, Capability, identifier, permissions

### Community 30 - "selah-desktop"
Cohesion: 0.80
Nodes (5): selah-application, selah-desktop, selah-domain, selah-infrastructure, selah-renderer

### Community 31 - "renderer/src/lib.rs"
Cohesion: 0.13
Nodes (31): TextRegion, anim_total(), animated_text_grows_into_the_settled_frame(), effective_anim(), escape(), font_family(), fonts(), Frame (+23 more)

### Community 33 - "application/src/lib.rs"
Cohesion: 0.08
Nodes (42): Connection, ai_line_limits_preserve_source_coverage_and_sections(), apply_analysis(), apply_analysis_with_lines(), BibleRepository, clear_with_media(), labels_and_accents_survive_split(), load_project() (+34 more)

### Community 34 - "App.tsx"
Cohesion: 0.16
Nodes (23): App(), detectRemote(), download(), hasWelcomed(), invoke(), load(), present(), rememberWelcome() (+15 more)

### Community 35 - "SongSearch"
Cohesion: 0.22
Nodes (5): SearchState, songQuery(), SongSearch, SongSearchCache, Candidate

### Community 36 - "MotionPicker.tsx"
Cohesion: 0.12
Nodes (17): A, B, DIRECTIONS, EASINGS, MotionEditor(), MotionOverrideEditor(), moves, select() (+9 more)

### Community 43 - "desktop-schema.json"
Cohesion: 0.40
Nodes (4): anyOf, description, $schema, title

### Community 44 - "control.rs"
Cohesion: 0.14
Nodes (21): ChaseSource, CommandOrigin, font_paths_are_portable(), FontAsset, legacy(), legacy_defaults_and_wire_format(), MidiBinding, MidiMap (+13 more)

### Community 45 - "types.ts"
Cohesion: 0.11
Nodes (29): applyMidiLearn(), MidiLearnEvent, DEFAULT_MIDI_MAP, DEFAULT_OSC_SETTINGS, DEFAULT_TIMECODE_SETTINGS, frames(), source(), validateControlContracts() (+21 more)

### Community 46 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 47 - "LayoutEditor.tsx"
Cohesion: 0.18
Nodes (22): desktop(), overflowRegions(), registered, registerFontAsset(), registerProjectFonts(), LayoutBuilder(), LayoutEditor(), Target (+14 more)

### Community 48 - "remote.rs"
Cohesion: 0.24
Nodes (16): Request, arg(), handle(), header(), json(), lan_address(), reply(), AppHandle (+8 more)

### Community 49 - "devDependencies"
Cohesion: 0.15
Nodes (13): devDependencies, playwright, @tauri-apps/cli, tsx, @types/react, typescript, vite, playwright (+5 more)

### Community 50 - "dependencies"
Cohesion: 0.22
Nodes (9): lucide-react, dependencies, lucide-react, react, react-dom, @tauri-apps/api, react, react-dom (+1 more)

### Community 51 - "listen"
Cohesion: 0.15
Nodes (16): MidiInputConnection, Device, devices(), listen(), raw(), AppHandle, Arc, Clock (+8 more)

### Community 52 - "package.json"
Cohesion: 0.29
Nodes (6): engines, node, name, private, type, version

### Community 53 - "build-guide-pdf.mjs"
Cohesion: 0.40
Nodes (4): html, pdf, result, root

### Community 54 - "prepare-dmg-extras.mjs"
Cohesion: 0.40
Nodes (4): dest, destDir, pdf, root

### Community 57 - "osc.rs"
Cohesion: 0.23
Nodes (13): OscMessage, OscPacket, bind(), emit(), parse(), poll(), Option, PresentationCommand (+5 more)

### Community 58 - "ContentDialog.tsx"
Cohesion: 0.33
Nodes (7): en, es, Modal(), cache, LinesPerBlock(), SongWebSearch(), Verse

### Community 59 - "capture-screenshots.mjs"
Cohesion: 0.27
Nodes (9): dismissWelcome(), main(), mime, outGuide, outReadme, outSite, root, serveDist() (+1 more)

### Community 60 - "files"
Cohesion: 0.22
Nodes (8): capturedAt, files, 01-present.png, 02-layout.png, 03-media.png, 04-settings.png, 05-library.png, 06-overview.png

### Community 61 - "Native control adapters (Wave 2)"
Cohesion: 0.40
Nodes (4): Invokes, Native control adapters (Wave 2), Protocol details, Validation

### Community 62 - "macOS-schema.json"
Cohesion: 0.40
Nodes (4): anyOf, description, $schema, title

### Community 63 - "Presentation command bus, Wave 1"
Cohesion: 0.50
Nodes (3): Persisted contracts and Wave 2 hooks, Presentation command bus, Wave 1, Tauri and LAN API

### Community 64 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

## Knowledge Gaps
- **243 isolated node(s):** `CommandOrigin`, `capturedAt`, `01-present.png`, `02-layout.png`, `03-media.png` (+238 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `main.rs` to `application/src/lib.rs`, `timecode.rs`, `display.rs`, `composite.rs`, `PresentationBlock`, `renderer/src/lib.rs`?**
  _High betweenness centrality (0.090) - this node is a cross-community bridge._
- **Why does `LiveSnapshot` connect `LiveSnapshot` to `application/src/lib.rs`, `main.rs`, `display.rs`, `composite.rs`, `PresentationBlock`, `renderer/src/lib.rs`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Why does `Runtime` connect `timecode.rs` to `main.rs`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Are the 25 inferred relationships involving `dispatch()` (e.g. with `analyze_lyrics()` and `apply_presentation_command()`) actually correct?**
  _`dispatch()` has 25 INFERRED edges - model-reasoned connections that need verification._
- **What connects `CommandOrigin`, `capturedAt`, `01-present.png` to the rest of the system?**
  _243 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Canvas.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Should `main.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.121765601217656 - nodes in this community are weakly interconnected._