# BellaNote

A local-first desktop companion for beautiful meeting notes. *Bella* is both a daughter’s name and the Italian for *beautiful*.

This repository is the greenfield home for the cross-platform product (macOS and Windows). Product definition lives in [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md). The first build slice (Mac hierarchy, file ingest, scoped ChatGPT) is specified as user stories in [`USER_STORIES.md`](./USER_STORIES.md). Interview decisions for the first POC are in [`INITIAL_BUILD_NOTES.md`](./INITIAL_BUILD_NOTES.md).

An earlier macOS proof of concept (Tauri 2, on-device Whisper, system-audio capture, summaries, and meeting chat) informed that scope and will be ported, not discarded.

## Run the desktop app (macOS)

```bash
npm install
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r requirements.txt
npm run tauri:dev
```

Homebrew’s Python is “externally managed,” so do **not** `pip install` into the system interpreter. The app uses `.venv/bin/python3` when that folder exists (or `ECHO_PYTHON` if you point it elsewhere).

Requires Node 20+, Rust (via rustup), and Xcode Command Line Tools. Add your OpenAI API token in Settings to use scoped chat.
