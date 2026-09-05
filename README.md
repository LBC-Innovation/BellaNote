# BellaNote

A local-first desktop companion for beautiful meeting notes. *Bella* is both a daughter’s name and the Italian for *beautiful*.

This repository is the greenfield home for the cross-platform product (macOS and Windows). Product definition lives in [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md). The first build slice (Mac hierarchy, file ingest, scoped ChatGPT) is specified as user stories in [`USER_STORIES.md`](./USER_STORIES.md). Interview decisions for the first POC are in [`INITIAL_BUILD_NOTES.md`](./INITIAL_BUILD_NOTES.md).

An earlier macOS proof of concept (Tauri 2, on-device Whisper, system-audio capture, summaries, and meeting chat) informed that scope and will be ported, not discarded.

## Run the desktop app (macOS)

```bash
npm install
pip install -r requirements.txt
npm run tauri:dev
```

Requires Node 20+, Rust (via rustup), Xcode Command Line Tools, and `faster-whisper` on `python3`. Add your OpenAI API token in Settings to use scoped chat.
