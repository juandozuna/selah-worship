# Sources and third-party components

UI research used the inspected Pitch editor reference through Lazyweb, plus [ProPresenter's interface](https://support.renewedvision.com/hc/en-us/articles/360041345954-Understanding-The-ProPresenter-User-Interface), [recognition over recall](https://www.nngroup.com/articles/recognition-and-recall/), and [progressive disclosure](https://www.nngroup.com/articles/progressive-disclosure/). Selah adapts these interaction patterns to a smaller lyric-focused workflow.

- [World English Bible](https://ebible.org/details.php?id=engwebp): public-domain wording; retain translation identity and source terms. The World English Bible name is a trademark of eBible.org.
- [Reina-Valera 1909](https://ebible.org/details.php?id=spaRV1909): public-domain Spanish wording, including its historical orthography.
- [Qwen3-4B-Instruct-2507](https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507): Apache-2.0 base model. The pinned [LM Studio community GGUF](https://huggingface.co/lmstudio-community/Qwen3-4B-Instruct-2507-GGUF) supplies Q4_K_M; its LFS SHA-256 is recorded in `assets/model.json`.
- [llama.cpp](https://github.com/ggml-org/llama.cpp): MIT runtime, packaged from official releases with digest verification.
- [NDI SDK](https://docs.ndi.video/all/developing-with-ndi/sdk): distributed separately; honor the vendor's SDK/runtime terms. Selah dynamically loads it.
- [Syphon](https://github.com/Syphon/Syphon-Framework): distributed separately; retain its framework copyright/license notices when bundling it.
- [Tauri](https://v2.tauri.app/start/), [wgpu](https://wgpu.rs/), [resvg](https://github.com/linebender/resvg), React, and SQLite: retain their respective distribution notices.
- Cloud adapters follow [OpenAI Chat Completions](https://developers.openai.com/api/reference/resources/chat), [Anthropic Messages](https://platform.claude.com/docs/en/api/messages/create), and [Gemini generateContent](https://ai.google.dev/api/generate-content). Search uses [Brave Search](https://brave.com/search/api/).

The sample hymn library uses public-domain English texts by John Newton, Reginald Heber, and Thomas Ken. It does not include recordings or musical arrangements. Imported and extracted songs retain their source and usage metadata; online discovery is not a promise of licensed catalog coverage.
