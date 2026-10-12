# Selah

A worship lyric presentation app for macOS and Windows. One shared React interface and Rust core; a transparent canvas feeds NDI or macOS Syphon receivers.

## Try the interface

Requires Node 22.9+ and npm. Installation and activation are separate: an existing terminal can still use an older Node after you install a newer one. Run `npm run doctor` to see the active version and executable. If you use nvm, `nvm use` selects the project's Node 22 version; `nvm use 23` also works if that is the version you already installed.

```sh
npm install
npm run dev
```

Open **http://127.0.0.1:5173** after Vite reports that it is ready. Keep the terminal running.

To open the UI from other devices on the same network, run `npm run dev:lan` (or `npm run preview:lan` for the production build on port 4173) and browse to `http://<this-computer-ip>:5173`. Vite prints the "Network" URL on startup. Only do this on networks you trust, and allow incoming connections if the OS firewall asks. To check the production web build:

```sh
npm run build
npm run preview
```

Open **http://127.0.0.1:4173**. Serve `dist/` through HTTP; opening `dist/index.html` directly can block its JavaScript modules. For a single file that can be opened directly, use the preview below.

For a preview without a server:

```sh
npm run preview:file
```

Open `artifacts/preview.html`. The browser preview supports local songs, paste imports, service ordering, lyric navigation, canvas design, portable service files, and browser-local autosave. Native outputs, Bible packages, AI and web search require the desktop app. Browser output is a preview, not an NDI/Syphon sender.

Allow browser storage for autosave. If stored content cannot be restored, Selah pauses autosave to preserve the saved copy. Preparation and export remain available; opening a valid service from **Service options** resumes autosave. A storage access or quota error is reported without crashing onboarding.

## Run the desktop app

Requires Rust stable and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/). Supported targets are Apple Silicon macOS 12+ and Windows x64, with 16 GB RAM recommended for local AI.

```sh
npm install
npm run desktop
```

The build hook downloads WEB and Reina-Valera 1909 from eBible.org and the platform's official llama.cpp runtime. Runtime downloads are checked against GitHub release digests. Network access is necessary for this initial preparation. Generated binaries and Bible archives are excluded from source control.

The desktop bundle includes llama.cpp and its native libraries; no separately running AI service is needed. In Settings, choose **Set up local AI** to install the pinned Qwen3 4B Q4_K_M model (about 2.5 GB) and start the worker. The first local analysis or connection test also installs the model automatically if needed. Its SHA-256 and GGUF signature are verified before installation; a completed download left by an interrupted setup is recovered without downloading it again. Subsequent analysis starts the managed local worker automatically and works offline. Closing the main Selah window or quitting the app stops and reaps its AI worker and active curl requests/downloads, including child processes, and prevents any new launches during shutdown. Closing only a physical display window keeps Selah running. Model files remain installed for the next launch. Forced termination or a crash bypasses this normal shutdown handling. Advanced settings permit a different executable and GGUF file. Optional OpenAI, Anthropic, Gemini, or compatible endpoints use your own key and model ID. Keys are stored in macOS Keychain or Windows Credential Manager; they are never exported with services.

Online lyric discovery uses a Brave Search API key. Web text is extracted as a candidate and must be reviewed. Unreadable or blocked pages fall back to opening the source and pasting its text. AI proposes section labels and source-line boundaries, and cannot insert or replace words.

## Media layers (standalone)

In **Layout**, add background media (behind lyrics) and foreground media (in front of lyrics). Images and videos are supported. For video, unmute and choose an audio output device when needed. The physical display always shows the composite.

In **Settings → Output**, enable **Send media on NDI / Syphon** to bake media into the published feed. Leave it off (default) for transparent lyrics into ProPresenter or another switcher.

## ProPresenter

Selah does not remote-control ProPresenter. Start NDI and/or Syphon in Settings, then select the Selah feed as an input/source in ProPresenter. See `docs/guide/Selah-Volunteer-Guide.pdf` for volunteer steps.

## Present a service

1. Choose **Add content** to select a saved song, paste text, import OpenLyrics, or load Scripture.
2. Click a prepared lyric block to put it on air. Use Previous/Next, arrows, or Space; Escape clears lyrics.
3. Open **Layout** to choose centered, lower-third, or bilingual presets. Drag regions or enter exact coordinates, then apply the layout. Changes to a live item stay off air until you present again.
4. Start NDI or Syphon in Settings and select the named feed in your receiver. Clear lyrics sends transparent frames while the feed stays active.

NDI requires an installed runtime exporting the standard NDI SDK functions. macOS searches the standard SDK and `/usr/local/lib` locations; Windows uses `NDI_RUNTIME_DIR_V6`, `NDI_RUNTIME_DIR_V5`, or the DLL search path. Syphon uses `Syphon.framework` from `/Library/Frameworks`, the user's frameworks folder, or the app's private frameworks. Syphon is unavailable on Windows. These runtime dependencies are reported if missing.

Bible queries include `John 3:16-18`, `Juan 3-4`, and `Génesis 1:31-2:3`. Additional translations can be imported as USFM or verse-per-line files, individually or in ZIP packages. Verse bridges such as `1-2` must be separated before importing. Paired translations match canonical book/chapter/verse IDs; missing counterparts are flagged for review. Users review bilingual lyric alignment block by block.

## Architecture

- `crates/domain`: serializable content, layout, project, and immutable live snapshot types; validation. No UI, database or SDK dependencies.
- `crates/application`: repository/output/credential ports and content, reference parsing, and presentation use cases.
- `crates/infrastructure`: SQLite migrations, imports, web requests, AI adapters, OS credential stores, and dynamic NDI binding.
- `crates/renderer`: shared resvg/rustybuzz text shaping and RGBA composition, uploaded to wgpu textures. Native previews use the same renderer as published frames, in a separate worker.
- `src-tauri`: desktop commands, asset bootstrapping, independent publishing thread, managed inference process, and the macOS Metal/Syphon bridge.
- `src`: shared desktop interface and a browser preparation adapter.

SQLite stores a versioned project and translation records in the app data directory. Opening a project restores preparation, but starts with clear output. Portable service imports retain this computer's AI/credential configuration. Secrets do not pass into webview state or localStorage.

## Validation and builds

```sh
npm test
npm run build
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml
npm run desktop:build
```

The desktop and renderer manifests are separate from the core workspace so the domain/application/infrastructure can be tested without installing desktop/GPU dependencies. CI validates both target operating systems. Signing requires the distributor's signing identities; no signed installer has been generated in this workspace.

**Verified in this update:** production frontend build and twelve TypeScript tests, including blocked storage, malformed saved data, and failed autosave preservation. Earlier checks covered nine Rust tests, core Clippy, macOS bridge syntax, and browser presenting, advancing, clearing, and layout editing. Live browser checks could not be repeated in this restricted session.

**Still needs external validation:** full Tauri/renderer compilation (uncached dependencies cannot be downloaded in this session), actual NDI/Syphon receivers and alpha/orientation checks, live model/provider/search calls, signed installers, the two-hour performance run, and first-time volunteer usability testing. The graphify update was attempted but its installed Python/tree-sitter environment is incompatible with this machine.

See [release acceptance checks](docs/acceptance.md) and [content/runtime sources](docs/sources.md).

## Desktop app with built-in network access

`npm run desktop:build` produces `src-tauri/target/release/bundle/macos/Selah.app` (and a `.dmg`). While the app is running it also serves the full UI on port 47832, so any device on the same network can open `http://<this-computer-ip>:47832` and operate it. The addresses are shown in Settings → Network access. Credential saving, model download and local-AI startup stay desktop-only. Anyone on the network can operate the app while it runs.


### Local AI setup and shutdown verification — October 10, 2026

The Apple Silicon desktop app and DMG were rebuilt successfully. Verified model recovery using the pinned SHA-256, restart persistence, and a real bilingual local analysis through Settings → Test connection. Both closing the main window and Quit terminated Selah and llama-server; no curl children remained and ports 47831/47832 were closed. All 17 Rust tests (including installer and process lifecycle tests) and 16 TypeScript tests passed. Core Clippy passed; desktop Clippy passed with the existing output Message large-enum warning excluded. Graphify's AST update completed with warnings that five configuration files produced no nodes. Windows runtime behavior and forced-termination cleanup remain unverified; normal exit handlers cannot cover an abrupt crash or forced kill.


### Live lyric translation

In **Present**, enable **Live AI translation** and choose the target language. Translation always runs on this computer with the installed Qwen model, regardless of the AI provider selected for lyric organization; it never calls a paid cloud API. Present a song to begin: the current block is translated first, then only the remaining blocks of that on-air song. Selecting a different song in the service list does not preload it. Repeated text is reused from a bounded session cache, with separate entries for each language and lyric revision. Translations are not written into the library or exported service.

Original lyrics remain visible during inference or a failure; translations appear in the second-language output region as they become available. A temporary bilingual layout is supplied when no secondary region exists, including the physical display's own layout. Disabling the toggle restores the original layout and any manually paired lyrics. Changing songs, clearing output, changing languages, or disabling translation replaces pending work; an already-running local request may finish, but its stale result cannot replace the current output. A song already in the target language needs no inference. Scripture continues to use the paired Bible translations. Local speed and translation quality depend on the computer and model; this is on-demand translation rather than an instantaneous speech-caption service.


Live translation validation on Apple Silicon macOS: enabled the toggle before presenting and confirmed no model worker started; presented Amazing Grace and verified all six blocks translated locally into Spanish, appeared alongside originals, and stayed out of the saved library and layout. Disabling the toggle restored original output. Quitting stopped the local model and closed its port. The 25 frontend tests, 17 core Rust tests, two installer tests, and six renderer tests passed (one renderer timing benchmark intentionally ignored). Desktop packaging and Clippy passed with the existing output Message large-enum warning excluded. Physical-display and NDI/Syphon receiver integration still require the acceptance checks above; translation wording in the other target languages has not been evaluated.


### Automatic song search and local preparation

Both **Add content → Songs** and **Library** check this computer's songs first. If a title/artist/lyric query has no local song match, Selah automatically searches Brave after a 600 ms pause (at least two characters). Successful web results are cached for five minutes, and identical concurrent searches share one request. A local match suppresses automatic online search; **Search online** remains available for an explicit alternative search. This uses the Brave key already saved in the OS credential store.

Choose **Lines per block** (1–8), then **Select & prepare song** beside a web result. Selah fetches the selected page, uses the local model to select its actual lyric lines while excluding page boilerplate, and automatically creates verse/chorus blocks. Local AI aims for the requested line count, with shorter blocks at section boundaries; the application enforces the chosen maximum even if the model groups too many lines. Original lyric lines are reconstructed from the source and preserved exactly, once, in order. The flow always uses local AI regardless of the provider configured for other AI tasks. Review the extracted lyrics and source details before **Add to service**. Changing the line selector in the prepared import automatically reorganizes those lyrics with local AI.

Some sources block automated fetching or require JavaScript and do not supply readable lyrics. In that case choose another result or paste the lyrics in **Paste or import**; the app never substitutes invented lyrics. This flow prepares a draft and does not automatically save or put a song on air.


Automatic search validation: 31 frontend tests and 20 core Rust tests pass, including stale-query cancellation, cache reuse, stalled search timeout, exact source reconstruction, mandatory section boundaries, and selected line caps. The two desktop installer tests also pass after fixing parallel fixture directory collisions. The desktop app/DMG and Clippy checks passed (excluding the existing output Message large-enum warning). Live local preparation was checked against a publishing source returned by the existing Lavado Elevation search. Live Brave verification reached the saved-key lookup but was waiting inside macOS Keychain; approve any Selah Keychain prompt on this machine before retrying. Search now reports a timeout after 30 seconds rather than spinning indefinitely during a credential or network wait.


After the user approved macOS Keychain access, the full desktop flow was verified: retried Lavado Elevation returned Brave results, selecting the Essential Music Publishing result automatically fetched and identified its lyrics, and local AI opened a review with 34 blocks at the selected two-line maximum. The user then resumed operating the app. The Keychain issue is resolved for this build.
