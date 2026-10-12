# Release acceptance

Run these on Apple Silicon macOS and Windows x64 reference machines with 16 GB RAM. A passing frontend/core suite does not establish production readiness.

## Desktop integration

- Build the desktop and renderer; package the app and inspect the bundled Bibles and llama.cpp executable.
- Import WEB and RV1909, confirm 66 canonical books and complete verse ranges, and verify source wording against eBible.org. Test accented book names, multi-chapter ranges, invalid references, and missing counterpart verses.
- Start NDI at 1920×1080 / 30 fps; select the named source in a standard NDI receiver. Verify straight alpha, readable accents, frame orientation, transparent clear, and receiver restart.
- On macOS, repeat with Syphon, then both protocols together. On Windows, Syphon must explain that it is macOS-only.
- Change dimensions and layout while output is clear; check exact dimensions in the receiver. While lyrics are live, edit text and layout and verify that the previous snapshot remains visible until the next presentation action.
- Load the test pattern; verify orientation and colors. Restart the app; output must start clear.

## Content and intelligence

- Import TXT, OpenLyrics entities/breaks, USFM inline formatting and footnotes, and ZIP translation packages. Invalid imports must not partially replace installed verses.
- Pair Spanish and English blocks; require review of unmatched blocks. Verify long Scripture verses for overflow and split them into readable blocks during review if necessary.
- Install the pinned model and validate the GGUF checksum; interrupt a download, try a wrong checksum, restart, and retry. No unverified model becomes active.
- Analyze a reviewed English/Spanish corpus. Require every lyric line exactly once, unchanged, in source order. Check verse/chorus labels, repeated choruses, awkward phrase breaks, malformed model JSON, and omitted/repeated source lines.
- Test each cloud provider using a model capable of returning the constrained grouping format. Provider failure must not trigger a different provider automatically.
- Search for a locally saved song and confirm no automatic Brave request. Search for an absent song, pause typing, and verify automatic results in both Songs and Library. Choose a line count from 1–8, select a result, and confirm extraction and local AI grouping happen automatically. Rapidly change queries and close the dialog during requests; stale results must not replace current work. Change the prepared line count and confirm every source lyric line stays present, unchanged, exactly once, with no block over the chosen limit. Test an inaccessible source and missing Brave credentials.
- Search online with Brave; review extracted pages with navigation, HTML entities, incorrect artists, blocked pages, and absent lyrics. Candidate text must not become live automatically.
- Start local AI, then close the main window while a physical display is open; confirm Selah, llama-server, its descendants, and active curl downloads exit and ports 47831/47832 are closed. Repeat with Quit. Close only the physical display and confirm Selah remains usable.
- Disconnect internet and verify saved services, installed Scripture, local AI, and manual preparation still work.

- Enable Live AI translation with no song on air: no model worker or requests should start. Present a song and confirm the active block translates first and originals stay visible. Verify the second language on the preview, physical display, NDI, and Syphon, with both existing bilingual layouts and a single-language layout.
- Change songs, rapidly advance/clear, change languages during inference, and disable the toggle: late results must not change the wrong block or bring cleared lyrics back. Confirm repeated choruses are cached, browsing Library triggers no translation, and saved lyrics/layouts stay unchanged. Test model errors and Retry.

## Reliability and accessibility

- Run playback for two hours while repeatedly searching, editing layouts, and analyzing lyrics. Measure change-to-receiver latency; target ≤150 ms at 1080p after renderer initialization. Confirm no output freezes, uncontrolled memory growth, or unintended live changes.
- Test keyboard-only navigation, visible focus, Escape clearing, modal focus containment, resizing at 960×640, and reduced-motion settings.
- Test autosave failure and malformed portable services; existing saved data must survive. A database newer than the supported schema must not be downgraded.
- Test at least five first-time volunteers: find a song, load a passage, present/advance/return, and clear lyrics. At least four must finish without coaching or unintended output. Record observed issues before release.
- Sign and notarize the macOS package and sign the Windows installer using distributor-owned identities. Test fresh installs and uninstall/reinstall without losing saved preparation.
