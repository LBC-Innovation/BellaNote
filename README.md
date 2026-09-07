<!-- @format -->

# BellaNote

A local-first Mac and Windows app for **beautiful meeting notes**. The promise is a record you can trust and actually want to reopen: organized files, an on-device transcript, playback when there is audio, and a scoped chat that answers from those notes — not from the internet at large.

This repository is the greenfield product. The first Mac slice was files-only; this app now records from the **microphone** and **system audio + microphone** on both macOS (ScreenCaptureKit) and Windows (WASAPI loopback). Summaries, task extraction, and a sharing backend are still later.

Long-term product definition: [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md)  
What the current app does, story by story: [`USER_STORIES.md`](./USER_STORIES.md)  
First-slice technical contract: [`INITIAL_BUILD_NOTES.md`](./INITIAL_BUILD_NOTES.md)

---

## What the product does

Meetings still force a false choice: listen well, or write everything down. Most AI note tools solve the artifact problem (you get a transcript) by putting a bot in the call or sending audio to the cloud.

BellaNote is a **companion you open for the meeting**, not a bot that joins the call.

- You file audio and existing transcripts into a simple tree: **Organization → Topic → Meeting group → Files**.
- You can **Record Meeting** from the microphone, or mix **system audio + microphone** so the file has what you heard and what you said.
- Audio is copied onto the machine and transcribed **on device** with Whisper (`small.en`). The original file in Downloads is never deleted.
- Imported Zoom / Teams transcripts (`.vtt`, `.srt`, `.txt`) become readable notes immediately — no audio required.
- You play audio against a static waveform, follow the matching transcript line, change speed, and search the text.
- You ask ChatGPT (`gpt-4o`) a question at a chosen **scope**: this file, this meeting group, this topic, or the whole organization. Only ready transcript text is sent. Audio bytes never leave the machine.

The Duke walkthrough is the north star for this slice: organize class material under **Duke → Competitive Strategies → Lecture Class 1**, drop in a posted lecture recording and a Zoom export, then ask _“What did we say about switching costs in September?”_ at topic scope.

### In this slice

| You can                                         | You cannot (yet)                      |
| ----------------------------------------------- | ------------------------------------- |
| Run a native Mac or Windows app with no account | Paste a YouTube URL                 |
| Build the org / topic / group tree              | Get auto summaries or extracted tasks |
| Record microphone (Mac and Windows)             | Share a library with someone else     |
| Record system + mic on macOS and Windows        | Move a file between meeting groups    |
| Import one or more audio files or transcripts   |                                       |
| Play audio, follow the transcript, search lines |                                       |
| Chat at org / topic / group / file scope        |                                       |
| Keep the OpenAI token in the OS credential store |                                       |

---

## How it works

BellaNote is a **Tauri 2** desktop app: a React frontend in a native window, with Rust owning files, SQLite, transcription, and chat.

```
┌─────────────────────────────────────────────────────────────┐
│  React 19 + Vite + Tailwind + shadcn  (src/)                │
│  Library tree · Files / transcript · Waveform · Chat        │
└──────────────────────────┬──────────────────────────────────┘
                           │ Tauri invoke / events
┌──────────────────────────▼──────────────────────────────────┐
│  Rust host  (src-tauri/)                                    │
│  SQLite · file copies · keychain · OpenAI · whisper worker · capture  │
└──────────────────────────┬──────────────────────────────────┘
                           │ stdin / stdout JSON
┌──────────────────────────▼──────────────────────────────────┐
│  scripts/transcribe_worker.py  (faster-whisper, small.en)   │
└─────────────────────────────────────────────────────────────┘
```

### Local-first storage

Everything BellaNote owns lives under Application Support. There is no recordings-folder picker.

```
~/Library/Application Support/com.bellanote.app/
  bella.db
  library/
    {artifact_id}/
      source.*          # copied upload, omitted for transcript-only imports
      transcript.txt
      segments.json
```

SQLite holds the tree, artifact metadata, and one chat thread per scope. File bytes are the source of truth on disk. Closing and reopening the app restores the library. Pane collapse and playback speed are remembered in `localStorage`; the last selected tree row is not.

### Transcription

1. **Import Meeting → Audio files** copies each file into `library/{id}/` and creates a row.
2. **Record Meeting → Microphone** (or **System audio** on a Mac) writes `source.wav` into the same layout and transcribes in chunks while you record.
3. The first job becomes **Importing** (or **Recording**) and runs `small.en` through a persistent Python worker. Extra imported files wait as **Pending**.
4. On success the row is **Ready** (quality shows the Whisper model). On failure it is **Failed** and can be retried. A quit mid-import or mid-record marks leftover in-flight rows failed so they are not stuck forever.
5. **Import Meeting → Transcript files** parses `.vtt` / `.srt` / `.txt` and marks the artifact Ready immediately (`Imported`).

Only one Whisper job holds the worker at a time. The UI does not stream partial text; you wait, then read the full transcript.

### Chat

Chat is a first-class pane, not a modal. Scope chips (**Org / Topic / Group / This file**) default from the tree selection and can be changed before send.

- Context is packed **newest groups first** up to about **110k characters** of ready transcript text.
- The composer shows how many ready files are in scope and how many were omitted.
- The model is told to answer only from those texts, cite title + timestamp, and say when the material does not contain the answer.
- There is **one thread per scope**. **New thread** replaces it; there is no archive list yet.
- No OpenAI token → Settings opens. Invalid token → the question stays in the composer.

### Interface

Three glass panes: **Library | Transcript | Chat**. Each can collapse to a labeled rail. Titlebar buttons focus Library only, Transcript only, or Chat only; widths animate. Settings lives in the same titlebar cluster. The empty middle strip is the window drag region so those buttons stay clickable under macOS overlay traffic lights.

---

## How to use it

### 1. Build the tree

1. Launch BellaNote.
2. In Library, create an **organization** (example: `Duke`).
3. Inside it, create a **topic** (`Competitive Strategies`).
4. Inside the topic, create a **meeting group** (`Lecture Class 1`).

You cannot add files until that path exists. Rename and delete live on each row’s overflow menu. Delete always asks for confirmation. A checkbox in that dialog can also delete the original audio file; leave it unchecked to keep the file on disk.

**Collapse all** folds every org and topic in the library. Click a row (not the menu) to expand or collapse that branch.

### 2. Add files

Select a meeting group, then **Import Meeting** or **Record Meeting**:

- **Audio files** — `wav`, `mp3`, `m4a`, `aac`, `ogg`, `flac`. One or many. Status goes Pending → Importing → Ready (or Failed).
- **Transcript files** — `.vtt`, `.srt`, `.txt`. Ready as soon as they parse.
- **Microphone** — captures the default input. Works on Mac and Windows.
- **System audio** — mixes what you hear with the microphone. macOS uses ScreenCaptureKit; Windows uses WASAPI loopback of the default playback device. Headphones avoid speaker echo into the mic.

One recording at a time. Stop is instant; trailing transcript chunks may still arrive. A short consent line is visible while recording.

Do not click an **Importing** row expecting the transcript; a toast asks you to wait. **Failed** rows have **Retry**.

The Files card lists title + original filename, date added, and quality. Pencil renames the title; trash deletes with confirm. **Select multiple** appears only while Files is expanded.

### 3. Read and play

Open a **Ready** (or Failed) row. The Transcript card shows **Audio** and/or **Transcript** chips for what that file actually has.

On audio:

- The waveform is the full meeting on one strip. Click or drag to seek.
- Under the waveform: **play / pause**, **Follow**, speed (**1x / 1.25x / 1.5x / 2x**), and playhead / total time.
- Follow highlights and scrolls the current transcript line. Turn it off to read without the list jumping.
- Speed is remembered across files.

Imported transcripts have no waveform.

### 4. Search the transcript

The **Search transcript** field sits under the lines. Typing filters immediately (case-insensitive). Clear restores the full list. Follow does not auto-scroll while a query is active. Clicking a visible timestamped line still seeks the audio.

### 5. Ask a question

1. Open **Settings** (titlebar) and paste an OpenAI API token. It is stored in the OS credential store (`com.bellanote.app` / `openai_api_key`), never in the repo.
2. Set scope to Org, Topic, Group, or This file.
3. Ask in plain language. Answers render as markdown with citations.

If the scope has no Ready transcripts, nothing is sent and a toast says so.

### 6. Focus the workspace

Use the titlebar layout buttons to show only Library, only Transcript, or only Chat. Collapse any pane from its own header to a rail; click the rail to expand it again.

---

## Develop

### Prerequisites

- **macOS 13+ on Apple Silicon**, with Xcode Command Line Tools, or **Windows 10+** with [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (the installer can download it) and the MSVC build tools for local Rust builds
- **Node 22+** (Release CI uses Node 24)
- **Rust** via [rustup](https://rustup.rs/)
- **Python 3** for `npm run tauri:dev` only. Installers bundle their own worker.

On macOS, Homebrew’s interpreter is “externally managed” — use a venv.

### First-time setup

```bash
npm install
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
python3 -m pip install -r requirements.txt
```

`requirements.txt` installs **faster-whisper** for local `tauri:dev`. Packaged builds freeze that worker with `npm run sidecar:whisper` (also run automatically before the installer is assembled). The first audio import downloads the `small.en` model into Application Support (macOS) or AppData (Windows); that can take a minute.

### Run the desktop app

```bash
npm run tauri:dev
```

That starts Vite at `http://localhost:1420` and opens the native Tauri window. Use the **window**, not the browser tab: file dialogs, the OS credential store, SQLite, and the whisper worker only exist in Tauri.

Useful scripts:

| Command             | What it does                         |
| ------------------- | ------------------------------------ |
| `npm run tauri:dev`      | Native app + Vite HMR                         |
| `npm run tauri:build`    | Production `.app` + `.dmg` on macOS (builds the worker) |
| `npm run tauri:build:windows` | Production NSIS installer on Windows |
| `npm run sidecar:whisper` | Freeze Python + faster-whisper into a sidecar |
| `npm run dev`            | Vite only (UI without Rust commands)          |
| `npm run build`          | `tsc` + production frontend                   |
| `npm run tauri`          | Tauri CLI (`build`, etc.)                     |

### Install a local build

**macOS** (after first-time setup):

```bash
npm run tauri:build
```

The installer lands at `src-tauri/target/release/bundle/dmg/`. Open the `.dmg`, drag **BellaNote** into Applications, then launch it from there.

**Windows:**

```bash
npm run tauri:build:windows
```

The NSIS installer lands under `src-tauri/target/release/bundle/nsis/`. Run it; WebView2 is downloaded if this PC does not already have it.

Python and faster-whisper are frozen into the app. The person who installs BellaNote does not install those dependencies. The first audio import downloads the `small.en` model into Application Support (macOS) or AppData (Windows); after that, transcription stays on device.

### GitHub Releases

The **Release** workflow (`.github/workflows/release.yml`) freezes the whisper worker, then builds an Apple Silicon `.dmg` plus a Windows NSIS installer and attaches them to a **draft** GitHub Release. Intel Macs are not supported.

1. Keep `version` in sync in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
2. Either push a tag that matches `tauri.conf.json` (`git tag v0.1.0-beta.1 && git push origin v0.1.0-beta.1`) or run **Actions → Release → Run workflow**. The draft tag is always `v` plus the version in `tauri.conf.json` (`v__VERSION__`).
3. When the macOS and Windows jobs finish, open the draft release (the **Releases** page, not “Create a new release”), download the installer that matches the machine, and publish when you are ready.

If the job cannot create a release, set the repo **Actions** workflow permission to **Read and write**.

### Signing and notarization

GitHub Releases are signed and notarized with a **Developer ID Application** certificate. Installers from the Releases page should open after dragging BellaNote into Applications.

Account, certificate, Keychain trust, and GitHub Actions secrets: [`documentation/APPLE_CODE_SIGNING.md`](./documentation/APPLE_CODE_SIGNING.md).

**Do not create empty Apple signing secrets.** Tauri treats a *present* `APPLE_CERTIFICATE` as “import this certificate,” even when the value is `""`.

Official reference: [Tauri macOS code signing](https://v2.tauri.app/distribute/sign/macos/).

### Environment

Packaged builds spawn the bundled `transcribe-worker` sidecar. `tauri:dev` uses the repo Python worker:

1. `ECHO_TRANSCRIBE_SIDECAR` if set
2. else the sidecar next to the app binary, or `src-tauri/binaries/transcribe-worker-<triple>` (`.exe` on Windows)
3. else (debug only) `ECHO_PYTHON` / `.venv/bin/python3` / `.venv/Scripts/python.exe` / `python3` / `python` plus `scripts/transcribe_worker.py`

Optional:

| Variable                   | Default                        | Purpose                                      |
| -------------------------- | ------------------------------ | -------------------------------------------- |
| `ECHO_TRANSCRIBE_SIDECAR`  | bundled `transcribe-worker`    | Override the frozen worker binary            |
| `ECHO_PYTHON`              | `.venv/bin/python3` (Windows: `.venv/Scripts/python.exe`) | Interpreter for `tauri:dev` |
| `ECHO_TRANSCRIBE_SCRIPT`   | `scripts/transcribe_worker.py` | Override the worker script in `tauri:dev`    |
| `WHISPER_MODEL`            | `small.en`                     | Model name passed to faster-whisper          |

### Repository layout

```
src/                    React app
  App.tsx               Three-pane shell, titlebar, pane animation
  components/           Library, workspace, chat, player, dialogs
  store/                Zustand: library + artifacts
  lib/api.ts            Tauri command wrappers
src-tauri/              Rust host
  src/commands.rs       Invoke surface
  src/db.rs             SQLite
  src/transcribe.rs     Whisper worker process
  src/chat.rs           Scoped gpt-4o
  src/artifacts.rs      Import / retry / delete
scripts/transcribe_worker.py
```

Frontend talks to Rust only through `src/lib/api.ts`. Add a new capability there plus a Tauri command and a permission under `src-tauri/permissions/` / `capabilities/default.json`.

UI kit is **shadcn + Tailwind 4 + Radix**. Keep new chrome in that system; do not import a second component library. Dark charcoal / mint glass is the only theme in this slice.

### Conventions

- **Do not clone the BellaNote2 POC UI.** That repo informed capture and the whisper worker. This app has its own shell.
- **Recordings land in the same meeting group as imports** (`source_type` `voice` / `system`). System + mic uses ScreenCaptureKit on macOS and WASAPI loopback on Windows.
- **Never delete the user’s original file.** Confirm every delete; remove only BellaNote’s copy.
- Chat must send **transcript text**, never audio.
- User stories in [`USER_STORIES.md`](./USER_STORIES.md) are rewritten to shipped behavior. If you change a user-facing flow, update that file in the same change.

### Related documents

| File                                                 | Use it for                                                   |
| ---------------------------------------------------- | ------------------------------------------------------------ |
| [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md)             | Vision, category bet, what comes after this slice            |
| [`USER_STORIES.md`](./USER_STORIES.md)               | Shipped acceptance and leftover stories (US-205, US-303–305) |
| [`INITIAL_BUILD_NOTES.md`](./INITIAL_BUILD_NOTES.md) | Interview-locked technical decisions for the first cut       |
