# Graph Report - LyricPassingPresentationSoftware  (2026-10-12)

## Corpus Check
- 73 files · ~63,468 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 901 nodes · 1941 edges · 57 communities (50 shown, 7 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 29 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Canvas.tsx
- main.rs
- scripts
- LiveSnapshot
- application/src/lib.rs
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
- description
- react-dom.d.ts
- SqliteStore
- App.tsx
- songSearch.ts
- MotionPicker.tsx
- check-node.mjs
- LiveTranslator
- desktop-schema.json
- PermissionEntry
- types.ts
- local
- LayoutEditor.tsx
- remote.rs
- devDependencies
- dependencies
- SongSearch
- package.json
- build-guide-pdf.mjs
- prepare-dmg-extras.mjs
- package-dmg.sh
- main.js

## God Nodes (most connected - your core abstractions)
1. `AppState` - 33 edges
2. `blocking()` - 29 edges
3. `LiveSnapshot` - 28 edges
4. `dispatch()` - 26 edges
5. `scripts` - 25 edges
6. `desktop()` - 23 edges
7. `invoke()` - 21 edges
8. `App()` - 19 edges
9. `PresentationBlock` - 17 edges
10. `ensure_local_worker()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `SyphonPublisher` --implements--> `OutputPublisher`  [EXTRACTED]
  src-tauri/src/output.rs → crates/application/src/lib.rs
- `analyze_lyrics()` --references--> `PresentationBlock`  [EXTRACTED]
  src-tauri/src/main.rs → crates/domain/src/lib.rs
- `import_openlyrics()` --references--> `PresentationBlock`  [EXTRACTED]
  src-tauri/src/main.rs → crates/domain/src/lib.rs
- `DisplayFrame` --references--> `LiveSnapshot`  [EXTRACTED]
  src-tauri/src/display.rs → crates/domain/src/lib.rs
- `DisplayState` --references--> `LiveSnapshot`  [EXTRACTED]
  src-tauri/src/display.rs → crates/domain/src/lib.rs

## Import Cycles
- None detected.

## Communities (57 total, 7 thin omitted)

### Community 0 - "Canvas.tsx"
Cohesion: 0.13
Nodes (28): Canvas(), clamp(), Drag, frameKey(), Guide, HANDLES, Mode, Rect (+20 more)

### Community 1 - "main.rs"
Cohesion: 0.15
Nodes (57): FnOnce, MediaAsset, OutputStatus, analyze_lyrics(), AppState, AudioDeviceInfo, bible_lookup(), bible_translations() (+49 more)

### Community 2 - "scripts"
Cohesion: 0.08
Nodes (25): scripts, assets, build, desktop, desktop:build, dev, dev:lan, dmg:extras (+17 more)

### Community 3 - "LiveSnapshot"
Cohesion: 0.07
Nodes (52): clear(), block_overrides_song_overrides_project(), CanvasLayout, default_fit(), DisplayOutput, LiveSnapshot, MediaAsset, MediaLayer (+44 more)

### Community 4 - "application/src/lib.rs"
Cohesion: 0.20
Nodes (20): ai_line_limits_preserve_source_coverage_and_sections(), apply_analysis(), apply_analysis_with_lines(), clear_with_media(), labels_and_accents_survive_split(), load_project(), normalized(), parse_passage() (+12 more)

### Community 5 - "model.ts"
Cohesion: 0.15
Nodes (20): hymn(), clearLive(), CUT_MOTION, DIRECTIONS, EASINGS, fold(), ORDERS, portableProject() (+12 more)

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
Cohesion: 0.10
Nodes (17): Content and intelligence, Desktop integration, Release acceptance, Reliability and accessibility, Sources and third-party components, Architecture, Automatic song search and local preparation, Desktop app with built-in network access (+9 more)

### Community 15 - "definitions"
Cohesion: 0.13
Nodes (14): anyOf, definitions, Number, Target, Value, description, anyOf, description (+6 more)

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
Nodes (13): properties, Identifier, description, oneOf, type, default, description, type (+5 more)

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

### Community 31 - "description"
Cohesion: 0.50
Nodes (4): default, description, type, description

### Community 33 - "SqliteStore"
Cohesion: 0.13
Nodes (22): Connection, BibleRepository, BibleVerse, bibleworks_aliases_map_to_canonical_books(), openlyrics(), Result, String, Vec (+14 more)

### Community 34 - "App.tsx"
Cohesion: 0.18
Nodes (23): App(), detectRemote(), download(), hasWelcomed(), invoke(), load(), present(), rememberWelcome() (+15 more)

### Community 35 - "songSearch.ts"
Cohesion: 0.31
Nodes (4): SearchState, songQuery(), SongSearchCache, Candidate

### Community 36 - "MotionPicker.tsx"
Cohesion: 0.15
Nodes (14): A, B, DIRECTIONS, EASINGS, MotionEditor(), MotionOverrideEditor(), moves, select() (+6 more)

### Community 43 - "desktop-schema.json"
Cohesion: 0.40
Nodes (4): anyOf, description, $schema, title

### Community 44 - "PermissionEntry"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

### Community 45 - "types.ts"
Cohesion: 0.14
Nodes (15): Frame, Plan, translationLanguages, translationLayout(), TranslationStatus, DisplayInfo, DisplayOutput, DisplayOverview (+7 more)

### Community 46 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 47 - "LayoutEditor.tsx"
Cohesion: 0.19
Nodes (21): desktop(), overflowRegions(), preset(), LayoutBuilder(), LayoutEditor(), Target, fullBleedLayer(), importMedia() (+13 more)

### Community 48 - "remote.rs"
Cohesion: 0.24
Nodes (16): Request, arg(), handle(), header(), json(), lan_address(), reply(), AppHandle (+8 more)

### Community 49 - "devDependencies"
Cohesion: 0.18
Nodes (11): devDependencies, @tauri-apps/cli, tsx, @types/react, typescript, vite, @tauri-apps/cli, tsx (+3 more)

### Community 50 - "dependencies"
Cohesion: 0.22
Nodes (9): lucide-react, dependencies, lucide-react, react, react-dom, @tauri-apps/api, react, react-dom (+1 more)

### Community 52 - "package.json"
Cohesion: 0.29
Nodes (6): engines, node, name, private, type, version

### Community 53 - "build-guide-pdf.mjs"
Cohesion: 0.40
Nodes (4): html, pdf, result, root

### Community 54 - "prepare-dmg-extras.mjs"
Cohesion: 0.40
Nodes (4): dest, destDir, pdf, root

## Knowledge Gaps
- **215 isolated node(s):** `name`, `version`, `private`, `type`, `doctor` (+210 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `main.rs` to `SqliteStore`, `LiveSnapshot`, `display.rs`, `composite.rs`, `PresentationBlock`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **Why does `LiveSnapshot` connect `LiveSnapshot` to `main.rs`, `application/src/lib.rs`, `display.rs`, `composite.rs`, `PresentationBlock`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **Why does `LocalWorker` connect `PresentationBlock` to `main.rs`, `ManagedProcess`?**
  _High betweenness centrality (0.035) - this node is a cross-community bridge._
- **What connects `name`, `version`, `private` to the rest of the system?**
  _215 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Canvas.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.12873563218390804 - nodes in this community are weakly interconnected._
- **Should `main.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1482153660012099 - nodes in this community are weakly interconnected._
- **Should `scripts` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._