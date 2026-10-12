# Graph Report - LyricPassingPresentationSoftware  (2026-10-10)

## Corpus Check
- 65 files · ~46,799 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 805 nodes · 1714 edges · 47 communities (43 shown, 4 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 27 edges (avg confidence: 0.8)
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
- ContentDialog.tsx
- ai.rs
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
- SongSearch
- MotionPicker.tsx
- check-node.mjs
- LiveTranslator
- desktop-schema.json
- PermissionEntry
- types.ts
- local

## God Nodes (most connected - your core abstractions)
1. `AppState` - 31 edges
2. `blocking()` - 27 edges
3. `dispatch()` - 26 edges
4. `LiveSnapshot` - 23 edges
5. `scripts` - 22 edges
6. `App()` - 19 edges
7. `invoke()` - 18 edges
8. `PresentationBlock` - 17 edges
9. `desktop()` - 16 edges
10. `ensure_local_worker()` - 15 edges

## Surprising Connections (you probably didn't know these)
- `SyphonPublisher` --implements--> `OutputPublisher`  [EXTRACTED]
  src-tauri/src/output.rs → crates/application/src/lib.rs
- `Playing` --references--> `MotionSettings`  [EXTRACTED]
  src-tauri/src/output.rs → crates/domain/src/lib.rs
- `analyze_lyrics()` --references--> `PresentationBlock`  [EXTRACTED]
  src-tauri/src/main.rs → crates/domain/src/lib.rs
- `import_openlyrics()` --references--> `PresentationBlock`  [EXTRACTED]
  src-tauri/src/main.rs → crates/domain/src/lib.rs
- `DisplayFrame` --references--> `LiveSnapshot`  [EXTRACTED]
  src-tauri/src/display.rs → crates/domain/src/lib.rs

## Import Cycles
- None detected.

## Communities (47 total, 4 thin omitted)

### Community 0 - "Canvas.tsx"
Cohesion: 0.15
Nodes (24): Canvas(), clamp(), Drag, frameKey(), Guide, HANDLES, Mode, Rect (+16 more)

### Community 1 - "main.rs"
Cohesion: 0.11
Nodes (68): LocalWorker, FnOnce, OutputStatus, PathBuf, Request, analyze_lyrics(), AppState, bible_lookup() (+60 more)

### Community 2 - "scripts"
Cohesion: 0.04
Nodes (46): lucide-react, dependencies, lucide-react, react, react-dom, devDependencies, @tauri-apps/cli, tsx (+38 more)

### Community 3 - "LiveSnapshot"
Cohesion: 0.08
Nodes (48): clear(), block_overrides_song_overrides_project(), CanvasLayout, DisplayOutput, LiveSnapshot, MotionOverride, MotionSettings, PresentationBlock (+40 more)

### Community 4 - "application/src/lib.rs"
Cohesion: 0.20
Nodes (20): ai_line_limits_preserve_source_coverage_and_sections(), apply_analysis(), apply_analysis_with_lines(), BibleRepository, labels_and_accents_survive_split(), load_project(), normalized(), parse_passage() (+12 more)

### Community 5 - "model.ts"
Cohesion: 0.15
Nodes (22): overflowRegions(), hymn(), preset(), LayoutBuilder(), Target, CUT_MOTION, DIRECTIONS, EASINGS (+14 more)

### Community 6 - "bundle"
Cohesion: 0.07
Nodes (29): icons/icon.png, app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+21 more)

### Community 7 - "NdiPublisher"
Cohesion: 0.13
Nodes (16): c_char, Connections, OutputPublisher, Library, NdiPublisher, c_void, Drop, Option (+8 more)

### Community 8 - "transition.rs"
Cohesion: 0.09
Nodes (32): add(), box_blur(), build(), compose(), dip_passes_through_transparent_and_fade_midpoint_blends(), direction(), ease(), every_transition_starts_on_from_and_ends_on_to() (+24 more)

### Community 9 - "display.rs"
Cohesion: 0.20
Nodes (18): AtomicU64, DisplayOutput, close_display(), display_frame(), DisplayFrame, DisplayInfo, DisplayOverview, DisplayState (+10 more)

### Community 10 - "ContentDialog.tsx"
Cohesion: 0.21
Nodes (11): en, es, Modal(), MotionEditor(), cache, LinesPerBlock(), SongWebSearch(), Block (+3 more)

### Community 11 - "ai.rs"
Cohesion: 0.16
Nodes (27): SearchCandidate, analyze(), analyze_with_lines(), apply_local_boundaries(), extract_lyric_text_local(), local_boundary_suggestions_cannot_omit_duplicate_or_reorder_lyrics(), organize_local(), parse_translation() (+19 more)

### Community 12 - "compilerOptions"
Cohesion: 0.12
Nodes (16): DOM, DOM.Iterable, ES2022, src, compilerOptions, esModuleInterop, jsx, lib (+8 more)

### Community 13 - "credentials.rs"
Cohesion: 0.28
Nodes (12): CredentialStore, Credential, FileTime, get(), c_void, Result, String, Vec (+4 more)

### Community 14 - "Selah"
Cohesion: 0.11
Nodes (15): Content and intelligence, Desktop integration, Release acceptance, Reliability and accessibility, Sources and third-party components, Architecture, Automatic song search and local preparation, Desktop app with built-in network access (+7 more)

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
Nodes (21): Connection, BibleVerse, bibleworks_aliases_map_to_canonical_books(), openlyrics(), Result, String, Vec, usfm() (+13 more)

### Community 34 - "App.tsx"
Cohesion: 0.24
Nodes (20): App(), desktop(), detectRemote(), download(), hasWelcomed(), invoke(), load(), present() (+12 more)

### Community 35 - "SongSearch"
Cohesion: 0.22
Nodes (5): SearchState, songQuery(), SongSearch, SongSearchCache, Candidate

### Community 36 - "MotionPicker.tsx"
Cohesion: 0.16
Nodes (13): A, B, DIRECTIONS, EASINGS, MotionOverrideEditor(), moves, select(), slider() (+5 more)

### Community 43 - "desktop-schema.json"
Cohesion: 0.40
Nodes (4): anyOf, description, $schema, title

### Community 44 - "PermissionEntry"
Cohesion: 0.67
Nodes (3): PermissionEntry, anyOf, description

### Community 45 - "types.ts"
Cohesion: 0.13
Nodes (14): Frame, Plan, translationLanguages, TranslationStatus, DisplayInfo, DisplayOutput, Live, MotionOverride (+6 more)

### Community 46 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

## Knowledge Gaps
- **192 isolated node(s):** `name`, `version`, `private`, `type`, `doctor` (+187 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `main.rs` to `transition.rs`, `display.rs`, `LiveSnapshot`, `SqliteStore`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Why does `LiveSnapshot` connect `LiveSnapshot` to `transition.rs`, `display.rs`, `application/src/lib.rs`, `main.rs`?**
  _High betweenness centrality (0.044) - this node is a cross-community bridge._
- **Why does `LocalWorker` connect `main.rs` to `ai.rs`, `ManagedProcess`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **Are the 17 inferred relationships involving `dispatch()` (e.g. with `analyze_lyrics()` and `bible_lookup()`) actually correct?**
  _`dispatch()` has 17 INFERRED edges - model-reasoned connections that need verification._
- **What connects `name`, `version`, `private` to the rest of the system?**
  _192 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `main.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10704225352112676 - nodes in this community are weakly interconnected._
- **Should `scripts` be split into smaller, more focused modules?**
  _Cohesion score 0.0425531914893617 - nodes in this community are weakly interconnected._