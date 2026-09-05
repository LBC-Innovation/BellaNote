# BellaNote — Initial POC build notes

**Status:** Interview locked for the first desktop POC  
**Date:** 5 September 2026  
**Demo bar:** A student can do the Duke walkthrough on a Mac: create **Duke → Competitive Strategies → Lecture Class 1**, add an audio file and a Zoom transcript, then ask ChatGPT a question at org, topic, group, or single-transcript scope.

These notes refine [`USER_STORIES.md`](./USER_STORIES.md) for *what we build first*. Stories still describe the product; this file is the technical contract for the first cut.

---

## 1. What this POC must prove

| In | Out |
|---|---|
| macOS desktop app launches like BellaNote2 (Tauri 2) | Windows |
| Strict tree: Organization → Topic category → Meeting group → Artifacts | Live microphone or system+mic capture |
| Upload audio → one-shot local `small.en` transcript | Real-time / chunked transcript UI |
| Import existing `.vtt` / `.srt` / `.txt` | YouTube ingest |
| Static SoundCloud-style waveform + click-to-seek on **audio** artifacts | Waveform while “recording” (there is no recording) |
| Scoped ChatGPT (`gpt-4o`) at all four levels | Waveform timed comments (next slice) |
| OpenAI token in Settings, macOS keychain | Re-transcribe / `base.en` fast path |
| Local SQLite + files under Application Support | Folder picker, summaries, tasks, sharing backend |

**Success walkthrough**

1. Launch BellaNote.  
2. Create organization `Duke`.  
3. Create topic `Competitive Strategies`.  
4. Create meeting group `Lecture Class 1`.  
5. Add `lecture.m4a` → wait for a full `small.en` transcript.  
6. Import `zoom-export.vtt`.  
7. Ask a natural-language question with scope set to the topic (or org / group / this file).  
8. Read a cited answer; click a source to open that artifact.  
9. On the audio artifact, see a **static** waveform and click it to seek playback. The imported VTT has **no** waveform.

---

## 2. Decisions from the interview

### 2.1 Follow BellaNote2’s technical path

- **Shell:** Tauri 2 + Rust host + React/TypeScript frontend. Same launch story (`npm run tauri dev` → native window).  
- **Local preserve:** SQLite for the library; audio and imported files copied into the app library (not left as loose pointers that break when Downloads is cleaned).  
- **Transcription:** Keep the persistent **faster-whisper** Python worker pattern from BellaNote2 (`transcribe_worker.py`, bundled runtime later).  
- **Secrets:** OpenAI API token in the **macOS keychain**, same idea as BellaNote2’s LLM key commands.  
- **Do not clone the whole UI.** Port guts (worker, sqlite, file copy, peaks if cheap). Leave Ant Design and the live scrolling waveform behind.

### 2.2 Capture vs files

- **This POC is files only.** Design the artifact model so a future `source = voice | system` recording can drop into the same meeting group, but do not build capture.  
- When recording returns: show **no waveform during capture**; after stop, generate the static tape. Live `base.en` + later `small.en` re-transcribe is that future path, not this one.

### 2.3 Whisper

| Situation | Model | UI |
|---|---|---|
| **This POC — uploaded audio** | `small.en` once | Progress / “Transcribing…” then the full transcript. No chunked live text. |
| **This POC — imported VTT/SRT/TXT** | none | Immediate Ready. |
| **Later — live record** | `base.en` while capturing | Fast, imperfect live text. |
| **Later — re-transcribe** | `small.en` | Offline / after the fact. Not in this POC. |

### 2.4 Waveform (SoundCloud direction, thin)

BellaNote2’s **live** `WaveformCanvas` (RAF + scrolling RMS history) and **playback tape** that **scrolls the window with the playhead** (`PlaybackTapeWaveform`, 120s sliding viewport) are the performance problem. We are not porting those behaviors.

**This POC**

- After an audio file is in the library, compute a **static peak tape** for the full duration (or a downsampled envelope that fits the width).  
- Draw the whole meeting on one strip. **The tape does not scroll** during playback.  
- A **playhead line** moves across the static tape (SoundCloud).  
- **Click (and drag) on the tape seeks.**  
- Transcript-only artifacts: no waveform, no playhead.  
- **Timed comments on the tape:** designed for, not built. Reserve a `waveform_comments` table (or skip the table until the next slice — see §5).

**Next slice (already agreed)**

- Pins / comments aligned to a time on the tape, SoundCloud-style.

### 2.5 Chat

- Model: **`gpt-4o`**.  
- Token: user-pasted in Settings, keychain-backed. Chat disabled until set.  
- Scopes: **Organization / Topic / Meeting group / This transcript.** Default = wherever you are in the tree.  
- Only **Ready** transcripts in scope are sent. Audio is never uploaded.  
- Honest answers: cite artifact + group + timestamp when present; say so when the material does not contain the answer.

**Context cap (recommendation — you asked us to pick)**

Do **not** stuff the entire org and hope. Do **not** hard-fail if there are 40 lectures.

- Budget a safe prompt size (start ~100–120k characters of transcript text, newest meeting groups first).  
- Show a **scope preview** before send: “12 ready · using 8 most recent · 4 omitted”.  
- Warn when truncated. User can narrow scope.  
- Never silently drop files without listing them.

That is the best mix of accuracy (model actually sees the text), UX (you know what was used), and performance (predictable request size).

### 2.6 Hierarchy

**Strict for this POC.** You cannot add a file until:

`Organization → Topic category → Meeting group`

No unfiled artifacts. Create the parents first (or in a short create flow that still writes all three rows).

### 2.7 Storage

**Hide the disk.** Everything lives under macOS Application Support for the app id (same family as BellaNote2’s `app_data_dir` + `echo.db` + `recordings/`). No recordings-folder picker in this POC.

Proposed layout:

```
~/Library/Application Support/com.bellanote.app/
  bella.db
  library/
    {artifact_id}/
      source.m4a          # copied upload, or omitted for import-only
      source.vtt          # imported transcript file, if any
      transcript.txt
      segments.json
      peaks.json          # static waveform envelope, audio only
```

SQLite holds the tree and artifact metadata. Files are the source of truth for bytes.

### 2.8 UI direction

- **New, simpler shell.** Not a restyle of BellaNote2’s three-column Ant Design workspace.  
- **Still a UI kit** — no hand-rolled component library.  
- **Recommended kit:** [shadcn/ui](https://ui.shadcn.com/) + Tailwind CSS + Radix. Best path to a custom **charcoal / mint-teal “liquid glass”** look (blurred panels, hairline borders, mint accent) without fighting Ant Design’s opinionated chrome.  
- Dark default. Library tree left, artifact + waveform + transcript center, chat as a first-class pane (not a modal).  
- BellaNote2 mint-on-slate tokens can inform color, not layout.

---

## 3. Data model (POC)

```
organizations
  id, name, created_at
  UNIQUE name (case-insensitive)

topic_categories
  id, organization_id, name, created_at
  UNIQUE (organization_id, lower(name))

meeting_groups
  id, topic_id, name, occurred_at, created_at
  Duplicate names allowed if dates differ

artifacts
  id, meeting_group_id, title, source_type,  -- audio_upload | transcript_import
  status,                                    -- queued | transcribing | ready | failed
  has_audio, original_filename, created_at, error_message
  transcript, segments_json                  -- or file-backed; keep a cached transcript column for chat

chat_threads
  id, scope_type, scope_id, messages_json, updated_at
```

**Delete (proposed, still matching USER_STORIES D4):** confirm when not empty; remove BellaNote copies only; never delete the user’s original file in Downloads.

---

## 4. What we port from BellaNote2 vs what we leave

| Port / reuse | Leave behind |
|---|---|
| Tauri 2 window + Rust commands pattern | Ant Design, MDX editor, Systems Control prompt farm |
| faster-whisper worker + `small.en` (POC already knows how to bundle) | Live `WaveformCanvas` / scrolling RMS history |
| SQLite via rusqlite, app data dir | Flat `meetings` table as the only object |
| Keychain for API token | Discovery chat as single-meeting-only (replace with scoped chat) |
| Peak extraction idea (`get_meeting_audio_peaks`) — simplify to a static envelope | Sliding 120s playback tape, latency-compensated playhead graph |
| File dialog plugin | ScreenCaptureKit / cpal capture, archive, tasks rail |

Start **greenfield in this repo**. Copy worker scripts and the smallest Rust helpers; do not drag the BellaNote2 frontend across.

---

## 5. Explicit next-slice (do not sneak in)

- SoundCloud **comments** on the waveform (data + pins + click-to-add)  
- Live mic / system+mic record  
- `base.en` live + `small.en` re-transcribe  
- Waveform during capture  
- Windows  
- Folder picker, YouTube, summaries, tasks, backend share

Schema courtesy: artifact ids and `has_audio` stay stable so comments can attach to `(artifact_id, time_ms, body)` later.

---

## 6. Suggested first implementation slices

1. Tauri app boots; empty liquid-glass shell; Settings token field (keychain).  
2. Strict tree CRUD + persist + browse.  
3. Copy audio into library; `small.en` one-shot; status + reader.  
4. Import VTT/SRT/TXT; no waveform.  
5. Static waveform + click-to-seek + playhead (audio only).  
6. Scope preview + `gpt-4o` chat at four levels; citations open the artifact.

---

## 7. Still assumed (say if wrong)

These were not asked as interview questions; they are the defaults we will use unless you object:

- Duplicate **org** or **topic** names in the same parent are blocked (case-insensitive).  
- Meeting group names may repeat if `occurred_at` differs.  
- Chat threads persist locally per scope.  
- Delete is confirm-only (no undo toast in this POC).  
- Video (`mp4`) is rejected; ask for audio.  
- UI kit is shadcn/ui + Tailwind, charcoal + mint glass.
