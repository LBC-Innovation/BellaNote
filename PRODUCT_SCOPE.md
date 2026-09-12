# BellaNote — Business Logic, Value Scope & Product Requirements

**Document status:** Working draft for platform requirements — Mac + Windows capture shipped  
**Audience:** Founder, product, and engineering  
**Date:** 5 September 2026  
**Last implementation review:** 6 September 2026  
**Repo:** [LBC-Innovation/BellaNote](https://github.com/LBC-Innovation/BellaNote)  
**Inputs:** Founder brief, existing BellaNote2 POC (`~/Documents/_DEV/_PERSONAL/BellaNote2`), [Granola](https://www.granola.ai/), and the 2026 AI meeting-notes category  
**First-slice stories:** [`USER_STORIES.md`](./USER_STORIES.md)

---

## 1. Executive summary

BellaNote is a cross-platform desktop companion that lets people **stay present in the room** and still leave with a complete, beautiful record of what mattered. The name is intentional: *Bella* is both a daughter’s name and the Italian for *beautiful*. The product promise is **Beautiful Note** — notes that are accurate, organized, and a pleasure to return to.

The category is crowded. [Granola](https://www.granola.ai/) proved that **bot-free, device-audio capture** is the right capture model for private and client-facing meetings. Otter, Fireflies, Fathom, Jamie, Krisp, Tactiq, Avoma, and Notion AI Meeting Notes have made transcription, summaries, and action items table stakes. Most of those products still force a trade-off:

| Typical trade-off | What users lose |
|---|---|
| A visible meeting bot | Trust, especially on 1:1s and client calls |
| “Just type and we will enhance it” | Presence — the user still has their head in the laptop |
| Cloud-only transcription | Privacy, offline use, and control of the recording |
| Notes without the audio | The ability to *hear* the moment again |
| One transcript, one author | Fidelity when two people captured the same meeting differently |

BellaNote takes a different bet:

1. **You do not have to write to get value.** Light notes are optional. The transcript, audience-aware summary, tasks, and chat should stand on their own.
2. **Capture stays on the device.** Microphone, system-audio + microphone, local files, and imported transcripts are first-class. Transcription always runs locally. A backend, when we add one, is for *sharing and management*, not for listening.
3. **Organization is flexible, not mandatory.** Date and time are required. Organization and topic category are independent, optional layers.
4. **Beauty and simplicity are product features**, not polish applied later. The POC already has a strong visual language (mint pulse on cool slate, dark/light themes, waveform-led workspace). We keep that standard and make it run well on **macOS and Windows**.

### What we already know from the POC

The BellaNote2 prototype is a **Tauri 2 + React + Rust** macOS app that already proves the hardest local loop:

- Record **voice memo** (microphone) or **system + mic** (ScreenCaptureKit, no virtual audio driver, no meeting bot)
- Transcribe **on-device** with a persistent faster-whisper worker
- Live transcript with quality scoring, playback, and waveform
- Import a `.vtt` transcript without audio
- Generate an executive summary, **enhance user notes** against the transcript, and **chat with a meeting** (Discovery)
- Local task list, archive (audio deleted, notes kept), offline-safe recording
- User-owned LLM keys (OpenAI / Anthropic) stored in the OS keychain

That is a real product core. The greenfield repo should **not** reinvent capture and transcription. It should productize that core, add the missing ingest and organization model, make Windows a first-class peer, and only then grow into shared workspaces and a broader meeting-management surface.

### MVP in one sentence

A beautiful Mac and Windows desktop app where a person can capture or import a conversation locally, tag it as little or as much as they want, and immediately get a trustworthy transcript, an audience-aware summary, extracted tasks, and a chat they can ask “what did we actually decide?”

### What success looks like

A user finishes a town hall, a 1:1, or a planning session, closes their laptop lid on the *meeting*, and opens BellaNote afterward — not during — to understand what was important, who owns what, and what to do next. They were in the conversation. BellaNote was in the background.

### What this repo has shipped (first Mac slice)

The greenfield app is **Mac and Windows**. Live microphone recording and system + mic capture are in on both platforms (ScreenCaptureKit on macOS, WASAPI loopback on Windows). Summaries and tasks are not. It proves the Duke walkthrough:

- Strict tree: **Organization → Topic category → Meeting group → Artifacts** (a refinement of §6.3’s optional-tag meeting model; unfiled org/topic remains the long-term product, not this slice)
- **Record Meeting** from the microphone, or system audio + microphone, into the selected meeting group
- Import audio (`wav` / `mp3` / `m4a` / `aac` / `ogg` / `flac`) → copy into Application Support → one-shot on-device `small.en`
- Extra audio uploads wait as **Pending**; only the job that holds the whisper worker shows **Importing**
- Import `.vtt` / `.srt` / `.txt` as ready transcripts (no audio required)
- Static waveform, click-to-seek, and Follow-highlight on audio artifacts
- Scoped `gpt-4o` chat at org / topic / meeting group / this file, with a ready-count preview and a ~110k character budget
- OpenAI token in the OS credential store (macOS Keychain / Windows Credential Manager)
- Three-pane charcoal / mint glass shell (Library | Transcript | Chat), with collapsible rails

Story-level status lives in [`USER_STORIES.md`](./USER_STORIES.md).

---

## 2. Product thesis

### The problem

Meetings still force a false choice: **listen well**, or **write everything down**. Head-down note-taking produces incomplete notes and a disengaged participant. Head-up listening produces a better meeting and a foggy memory. Existing AI tools mostly solve the *artifact* problem (you get a transcript) without solving the *engagement* problem (you still feel you must monitor the tool). Granola’s own loop — type rough notes, then enhance — still asks the user to write during the call.

### The BellaNote answer

BellaNote is a **local-first meeting companion**, not a meeting bot and not a notepad that requires authorship in real time.

- During the meeting: one click to capture. Optional scratch notes if the user wants them. No bot in the attendee list.
- After the meeting: a beautiful workspace — transcript, playback, summary for the right audience, assigned tasks, and chat.
- Over time: meetings sit in a simple hierarchy (date always; organization and topic when useful) so later questions can draw on related conversations.

### Brand and design principles

These are constraints, not aspirations.

1. **Beautiful Note.** Typography, spacing, motion, and color should feel considered. The POC’s mint-on-slate language is the starting point, not a placeholder.
2. **Simple enough to start in ten seconds.** New meeting → record or import → done. Tags and AI are available, never blocking.
3. **Present over performative.** The UI during capture is calm: a meter, a timer, a live transcript the user can ignore.
4. **Local by default, shared by choice.** Audio never has to leave the machine. Sharing is an explicit later act.
5. **Honest AI.** Cite the transcript. Prefer “I don’t see that” over invention. Show what the user wrote versus what the model added.
6. **Cross-platform without a lowest-common-denominator UI.** Same product on Mac and Windows; native capture backends underneath.

### Who it is for (MVP)

Primary: knowledge workers, consultants, managers, and founders who live in back-to-back meetings on a laptop and care about privacy and craft.

Secondary (near-term): small teams who want a shared meeting record without inviting a bot.

Not yet: large enterprise sales floors that need Gong-class conversation intelligence, CRM writeback, and coaching scorecards. Those are later, if ever.

---

## 3. What the POC already proved

Treat the following as **inherited product truth**, not a rewrite.

| Area | POC capability | Implication for the new app |
|---|---|---|
| Capture | `voice` (mic via cpal) and `system` (ScreenCaptureKit mix of display audio + mic on macOS; WASAPI loopback + mic on Windows) | Keep both modes. Native backends; shared mixer. |
| Transcription | Persistent Python faster-whisper worker; model loads once per session; bundled runtime for release | Keep on-device transcription. Do not move this to a server for MVP. |
| Storage | Local SQLite meetings; audio on disk; archive drops audio and keeps text | Continue local-first. Design schema now for org, topic, attendees, purpose, meeting type. |
| Library | Grouped by today / yesterday / older days | Date grouping stays. Add optional filters for org and topic. |
| Transcript | Segment timestamps, quality tiers (`avg_logprob`), click-to-seek playback | Keep. Quality gating for LLM input is a real differentiator. |
| Import | WebVTT only | Expand to Teams/Zoom/Otter exports and plain text. |
| AI | Executive summary, enhance-notes, Discovery chat; customizable prompts; offline disables LLM only | Keep all three. Add meeting-type templates and task extraction. |
| Tasks | Manual local list, optional meeting link | Keep the rail. Add AI-proposed tasks the user confirms. |
| Settings | Recordings folder, LLM provider/model/key, prompt editors, min transcript quality, theme | Carry forward. Add Windows permissions and consent copy. |
| Design | Dark default, light theme, three-column shell (library / workspace / tasks) | Preserve the shell. Simplify where the POC accumulated settings density. |

**POC gaps that the new product must close**

The greenfield first slice closed the starred items for Mac files. The rest remain.

- ★ Windows capture path (mic + WASAPI system audio, shipped)
- No YouTube ingest
- ★ Local audio-file ingest (shipped)
- ★ Organization, topic category, and meeting group (shipped as a required tree, not optional tags)
- No attendees or purpose fields
- ★ Chat across org / topic / group / one file (shipped; BellaNote2 Discovery was single-meeting)
- Tasks are not derived from the conversation
- Summaries are not in this slice
- No calendar, no speaker labels, no sharing

---

## 4. Competitive landscape

### 4.1 Granola — closest conceptual peer

Granola is the product to study, not copy. From [granola.ai](https://www.granola.ai/) and their 2026 plan documentation:

- Bot-free capture from computer audio; works with Zoom, Meet, Teams, huddles, and in-person
- Calendar brief before external meetings (who is coming, last discussion, open threads)
- Human notes + silent transcript → “Enhance notes”; user text stays distinct from AI additions
- 29+ templates / Recipes (1:1, discovery, pipeline review, follow-up email, PRD, coaching)
- AI chat across a meeting or a folder of meetings
- Private by default; share when you choose; shared folders
- MCP so other AI tools can query notes
- People & Companies relationship view
- File upload as extra meeting context (beta)
- macOS, Windows, iOS, Android; Apple Watch announced
- Free tier limited by history (rolling ~30 days or a 25-note cap, depending on reporting); paid from roughly $14/user/month; Enterprise adds SSO and admin controls

**Granola weaknesses BellaNote can exploit**

- The core loop still rewards typing during the meeting. BellaNote’s thesis is the opposite: *be engaged; we will write the beautiful note.*
- Granola does not keep audio. BellaNote should, locally, so users can replay and seek.
- Reviews consistently call out Google-first calendar/login friction. BellaNote should treat Outlook and Google as equals, and should work with **no calendar at all**.
- Speaker attribution is weak past a few voices. We should be honest in MVP (optional labels) and invest later.
- History and raw transcripts are monetized by lock-in. BellaNote’s local library is the user’s, forever.
- No first-class YouTube, local-file, or third-party transcript import.

### 4.2 Other competitors and what users have learned to want

| Product | Capture model | What users like | What we should learn |
|---|---|---|---|
| **Otter** | Bot + live captions | Real-time shared transcript, collaborative highlight | Live transcript is useful *if ignorable*. Do not make it the center of the meeting. |
| **Fireflies** | Bot, plus desktop / extension options | Team archive, CRM fill, “AskFred” across months of calls, analytics | Cross-meeting Q&A and team library are expected once we share. CRM is later. |
| **Fathom** | Bot and bot-free | Generous free tier, instant summary, 90-second highlight clips | Instant “meeting over → notes ready” feeling. Clip/share is a later delight. |
| **Jamie** | Bot-free desktop | Speaker memory across recurring meetings, consent emails, templates | Speaker memory and consent notices are trust features, not extras. |
| **Krisp** | Bot-free + noise cancellation | Cleaner audio, accent tools, AI notes | Audio quality *is* transcript quality. Consider noise handling on Windows. |
| **Tactiq** | Browser captions, no bot | Live transcript, reusable prompts, screenshots into notes | Saved prompts / recipes. Screenshot-into-meeting is a later meeting-management idea. |
| **Notion AI Meeting Notes** | System audio inside Notion | Notes land where work already lives | Export and, later, push to Notion/Slack. Do not become a workspace suite in MVP. |
| **Avoma / Gong** | Bot + conversation intelligence | Coaching, talk ratios, CRM automation | Out of MVP scope. Do not chase sales enablement. |
| **Circleback** | Bot / automation | Closes the loop after the meeting (tasks actually move) | Task extraction is only valuable if the user can act on it. |
| **tl;dv / Bluedot** | Bot / video | Clips, chapters, CRM | Video is explicitly out of MVP. Audio + transcript is enough. |

### 4.3 Features users now expect (even if we sequence them)

From the category, users have been trained to want:

- Bot-free capture that works on every meeting app
- Calendar-aware start (title, attendees, one-click record)
- Meeting-type templates
- Action items with owners and dates
- Follow-up email draft
- Chat with one meeting and with a set of related meetings
- Search that actually finds a phrase from three weeks ago
- Share a note without sharing the raw audio
- Multi-language
- Consent / “this is being transcribed” hygiene
- Export to markdown, doc, or the tool they already use

BellaNote does **not** need all of these in MVP. It does need to look like it belongs in this category on day one, then win on **presence, local control, ingest flexibility, beautiful organization, and (later) multi-contributor transcript fidelity**.

### 4.4 Differentiation map

```
                    Must type during meeting
                              ▲
                    Granola   │
                              │
   Cloud / bot ───────────────┼─────────────── Local / no bot
   Otter, Fireflies, Fathom   │   BellaNote (target)
   Gong, Avoma                │   Jamie, Krisp
                              │
                              ▼
                    Works if you write nothing
```

**BellaNote’s unique wedges**

1. **Presence-first** — valuable with zero in-meeting typing.
2. **Four ingest paths** — live capture, YouTube, local audio, imported transcript.
3. **Optional hierarchy** — org and topic independent; date required.
4. **Local audio kept** — replay, seek, re-transcribe, quality inspection.
5. **Transcript fidelity as a team sport** (post-MVP) — multiple people upload their local transcripts of the *same* meeting; the system helps reconcile them. Almost nobody sells this.
6. **Path to a meeting workspace** — handwritten notes, related documents, not just a notepad.

---

## 5. Feature table

Priority: **P0** ships in MVP. **P1** follows immediately after a usable Mac + Windows loop. **P2** is the collaboration backend. **P3** is meeting-management expansion.

Origin: **Brief** = founder request. **POC** = already proven. **Granola** = category peer. **Market** = common competitor capability. **Diff** = BellaNote-specific.

| ID | Feature | What it does | Horizon | Priority | Origin | Notes |
|---|---|---|---|---|---|---|
| C1 | Microphone recording | Capture a voice memo / in-person conversation from the device mic | MVP | P0 | Brief, POC | **Shipped** (`voice` via `cpal` on Mac and Windows). |
| C2 | System + microphone capture | Capture meeting playback *and* the user’s voice, no bot | MVP | P0 | Brief, POC, Granola | **Shipped** on macOS (ScreenCaptureKit audio + `cpal` mic) and Windows (WASAPI loopback of the default playback device + `cpal` mic), shared mixer. |
| C3 | YouTube URL ingest | User pastes a YouTube link; app fetches audio and transcribes locally | MVP | P0 | Brief | Personal-use helper. Must show ToS/copyright notice. Fail gracefully on restricted videos. |
| C4 | Local audio file ingest | User picks wav/mp3/m4a/ogg/etc.; transcribe locally | MVP | P0 | Brief, Market | **Shipped** multi-file Import Meeting → Audio. MP4 via **Video files** (extract audio + keep video for Watch). |
| C5 | Import third-party transcript | Ingest VTT/SRT/TXT/DOCX from Teams, Zoom, Otter, etc.; audio optional | MVP | P0 | Brief, POC | **First slice:** `.vtt` / `.srt` / `.txt` only (2 MB cap). No DOCX. |
| C6 | Re-transcribe | Re-run local model on kept audio (better model, or after edit) | Near-term | P1 | POC | POC already has regenerate. Keep it. |
| C7 | Live transcript (ignorable) | Segments appear while recording; user is not required to watch | MVP | P0 | POC, Otter | **Shipped:** chunks update the artifact while capturing; Files shows Recording. |
| C8 | Local audio retention + playback | Keep audio on disk; waveform; click transcript to seek | MVP | P0 | POC, Diff | **First slice:** static full-width tape, playhead, click-to-seek, Follow highlight. No archive yet. Recordings use the same `source.wav` path. |
| C9 | Transcript quality display | Show low-confidence segments; optional filter before LLM use | MVP | P0 | POC, Diff | **First slice:** Files table shows model / Imported / Importing / Recording / Pending / Failed — not per-segment confidence. |
| M1 | Meeting title | Editable name; default from time or first words | MVP | P0 | Brief, POC | **First slice:** meeting-group name + artifact title (inline rename). |
| M2 | Attendees | Free-form people list; optional later contact pick | MVP | P0 | Brief | Not in POC. No directory required in MVP. |
| M3 | Meeting purpose | Short “why we met” field | MVP | P0 | Brief | Manual; AI may suggest after transcript exists. |
| M4 | Meeting type | Town hall, planning, 1:1, general, custom | MVP | P0 | Brief, Granola | Drives summary template. |
| M5 | Date and time | Always set; defaults to capture start; user-editable | MVP | P0 | Brief | **First slice:** meeting group `occurred_at` defaults to now and is shown, not edited. Artifact “date added” is `created_at`. |
| M6 | Organization (optional) | User-defined org / client / company; not required | MVP | P0 | Brief | **First slice:** required top-level container. Long-term: optional / independent of topic. |
| M7 | Topic category (optional) | User-defined topic; not required; independent of org | MVP | P0 | Brief | **First slice:** required under an org. Long-term: optional / independent of org. |
| M8 | Library by date | Today / yesterday / older, as in the POC | MVP | P0 | POC | **First slice:** tree library, not date buckets. Meeting groups still store `occurred_at`. |
| M9 | Filter / browse by org or topic | Narrow the library without forcing a tree | MVP | P0 | Brief | Hierarchy is a *view*, not a prison. Empty org/topic is valid. |
| M10 | Archive | Remove from active list; optionally drop audio | MVP | P0 | POC | Keep. |
| M11 | Full-text search | Search titles, transcript, notes, attendees | Near-term | P1 | Market | Users will ask for this immediately after MVP. |
| A1 | Audience-aware summary | Summary shaped by meeting type and chosen audience | MVP | P0 | Brief, POC | Evolve the existing executive-summary prompt into templates. |
| A2 | Enhance optional notes | If the user jotted anything, expand it from the transcript | MVP | P0 | POC, Granola | Keep the “your words vs AI words” distinction. |
| A3 | Assigned-task extraction | Propose owner + task + due date when spoken; user confirms | MVP | P0 | Brief, Market | Write into the tasks rail. Never silent-create. |
| A4 | Chat with this meeting | Request/response Q&A grounded in the transcript, with time cites | MVP | P0 | Brief, POC | **First slice:** This-file scope + markdown answers. Model cites `[00:17]` in prose; click-to-seek from a cite is not built. |
| A5 | Chat across related meetings | If org and/or topic is set, retrieve sibling meetings as extra context | Near-term | P1 | Brief, Granola | **First slice pulled this forward:** Org / Topic / Group scopes with a character budget and omitted-count preview. |
| A6 | Follow-up email draft | One-click draft recap + asks, copy or mailto | Near-term | P1 | Granola, Market | High perceived value, low build cost after A1. |
| A7 | Decision log | Extract “we decided X” as a first-class list | Near-term | P1 | Diff, Market | Complements tasks. Town halls and planning sessions especially. |
| A8 | Custom / saved prompts (“Recipes”) | User-saved post-meeting actions (PRD, coaching note, standup recap) | Near-term | P1 | Granola, Tactiq | POC already has editable system prompts; productize as recipes. |
| A9 | User-owned LLM | BYO key, provider + model picker, keychain storage | MVP | P0 | POC, Diff | **First slice:** OpenAI token in keychain, `gpt-4o` only. No Anthropic / model picker yet. |
| A10 | Offline-safe core | Record and transcribe offline; AI disabled with a clear explanation | MVP | P0 | POC | **Shipped:** record and transcribe work offline; chat needs the network and says so. |
| X1 | Copy / export transcript and notes | Markdown, plain text, VTT | MVP | P0 | POC, Market | POC exports timestamped text. Add markdown notes. |
| X2 | Light / dark theme | Ship both; dark default | MVP | P0 | POC | **First slice:** dark charcoal / mint glass only. |
| X3 | Recording consent reminder | Soft banner: you are capturing audio; know your jurisdiction | MVP | P0 | Jamie, Market | Trust. Not a legal product. |
| X4 | Calendar detect (optional) | Suggest title, attendees, time from Google or Outlook | Near-term | P1 | Granola, Market | Must work with *no* calendar. Do not block MVP on this. |
| X5 | Speaker labels | Manual rename of Speaker 1/2; later voice memory | Near-term | P1 | Jamie, Market | Auto-diarization is imperfect; ship manual first. |
| X6 | Multi-language transcription | Beyond `base.en` | Near-term | P1 | Granola, Market | Whisper already can; productize model choice. |
| B1 | Account and workspace | Sign-in; personal library remains local | Backend | P2 | Brief | Recording still local. |
| B2 | Share a meeting (notes + transcript, not audio by default) | Invite by email; permissions: view / comment / contribute | Backend | P2 | Brief, Granola | Audio upload is opt-in and off by default. |
| B3 | Multi-contributor transcripts | Same meeting, multiple local transcripts uploaded; compare / merge for fidelity | Backend | P2 | Brief, Diff | Signature collaboration feature. |
| B4 | Shared orgs and topics | Team taxonomy with personal overrides | Backend | P2 | Brief | Keep the independent-optional rule. |
| B5 | Shared task list | Tasks visible to the workspace; still user-confirmed | Backend | P2 | Brief | |
| B6 | Version history of notes | Who changed the summary, when | Backend | P2 | Market | |
| F1 | Handwritten note capture | Photograph / import Rocketbook-style pages; OCR; attach to meeting | Future | P3 | Brief | Meeting workspace, not just audio. |
| F2 | Related documents | Attach decks, agendas, emails as extra inference context | Future | P3 | Brief, Granola | Granola already started here. |
| F3 | Cross-artifact chat | Chat uses transcript + handwriting + docs + related meetings | Future | P3 | Brief | The “meeting memory” end-state. |
| F4 | Mobile capture | Phone for walking meetings and calls | Future | P3 | Granola | Desktop remains the system of record. |
| F5 | MCP / external AI access | Let Claude, ChatGPT, Cursor query *local* or shared notes | Future | P3 | Granola | Attractive once the library is rich. |
| F6 | Integrations | Notion, Slack, mail, later CRM | Future | P3 | Market | Export-first until then. |

### MVP cut line

**In:** C1–C5, C7–C9, M1–M10, A1–A4, A9–A10, X1–X3.

**Out of MVP on purpose:** calendar, speaker memory, YouTube-at-scale robustness, backend sharing, handwriting, MCP, CRM, video, mobile.

### First-slice ship status (this repo, Mac)

Pulled forward from the later table: **A5** (scoped chat across org / topic / group) shipped in the files-only slice.

| Shipped | Partial | Not in this slice |
|---|---|---|
| C4, C5 (vtt/srt/txt), C8, A4, A5, A9, A10 (transcribe offline), M1, M5–M7 (as a required tree), X2 (dark only) | C9 (model/status badge, not segment confidence), A4 cite-to-seek, A5 file-list preview | C1–C3, C6–C7, M2–M4, M8–M11, A1–A3, A6–A8, A10 offline-record modal, X1, X3–X6, all B/F |

---

## 6. Product scope (for platform requirements)

This section is the contract for the first requirements and architecture pass.

### 6.1 Product shape

- **Form factor:** Native desktop app for **macOS 13+ (Apple Silicon)** and **Windows 10+** (WebView2). Intel Macs are not supported.
- **Stack direction (recommended):** Continue **Tauri 2 + Rust + React + TypeScript**. The first slice and BellaNote2 already paid the tax on Whisper, keychain, and a dense desktop UI. Electron would fight the “efficient on both platforms” goal.
- **Local data:** SQLite + files under Application Support (`com.bellanote.app`). A user-chosen recordings directory is later.
- **AI:** Optional, user-configured. No AI vendor account required to record or transcribe.
- **Network:** Required only for LLM calls, YouTube ingest, and (later) sharing.

### 6.2 Capture and ingest

**First slice (shipped):** the user creates **Org → Topic → Meeting group**, then **Import Meeting** (audio or transcript) or **Record Meeting** (microphone, or system + mic) into that group. Multiple artifacts live in one group. Windows uses WASAPI loopback for system audio.

**Target product:** every meeting is created the same way: **New meeting** (or “drop something on a meeting”). Then the user chooses an ingest path.

```
                    ┌─ Microphone only
                    ├─ Microphone + system / meeting audio
New meeting ────────┼─ YouTube URL
                    ├─ Local audio file
                    └─ Transcript file (no audio)
```

**Requirements**

- One recording at a time.
- System+mic must capture **what the user hears** and **what the user says**, mixed into one timeline, without a virtual cable product.
- Permissions are explicit and recoverable: Microphone; on macOS also Screen Recording; on Windows the loopback/audio privacy prompt.
- YouTube: paste URL → resolve title → download audio to the meeting folder → transcribe locally. Surface legal copy: intended for meetings/talks the user is entitled to use; BellaNote does not bypass private or age-gated content.
- File ingest: common containers (wav, mp3, m4a, aac, ogg, flac, webm). MP4 via Import Meeting → Video files (keep video, extract audio once for transcript/playback).
- Transcript import: VTT, SRT, plain text, and at least one Teams/Zoom export shape. Preserve timestamps when present. `has_audio = false`.
- Stop recording is instant; trailing whisper chunks may still arrive (POC already handles this).
- User can rename, retag, and run AI after any ingest path. AI features do not care how the transcript arrived.

**Non-goals for capture**

- Video recording or gallery view
- A BellaNote bot that joins Zoom/Meet/Teams
- Automatic record-every-calendar-event (dangerous and consent-hostile)

### 6.3 Meeting model

**First slice (shipped):** the atomic user object is a **meeting group** that holds many **artifacts**. Required path: Organization → Topic → Meeting group → Artifact. Duplicate org/topic names in the same parent are blocked; meeting-group names may repeat. Date (`occurred_at`) is stored and shown, not edited.

**Target product:** a meeting is the atomic object. Organization and topic become optional independent tags again (see hierarchy rules below). The first-slice meeting group should map forward as “a meeting that can have many artifacts.”

| Field | Required | Default | Notes |
|---|---|---|---|
| `id` | yes | uuid | |
| `title` | yes | “Meeting, 5 Sep 2026 9:04” | User-editable |
| `started_at` / `ended_at` | yes | capture times or file mtime | User-editable |
| `organization_id` | no | none | Soft FK to user-defined org |
| `topic_id` | no | none | Soft FK to user-defined topic |
| `attendees` | no | empty | List of display names |
| `purpose` | no | empty | One or two sentences |
| `meeting_type` | no | `general` | Enum + custom label |
| `transcript` + `segments` | no | empty until ingest | |
| `notes` | no | empty | User markdown |
| `summary` | no | empty | Generated, then editable |
| `source` | yes | `voice` / `system` / `youtube` / `file` / `import` | |
| `has_audio` | yes | | |
| `is_archived` | yes | false | |

**Hierarchy rules (founder constraint — target product)**

- Date/time is always present.
- Organization and topic are **independent**. Valid states: neither, org only, topic only, both.
- The UI may *display* as Org → Topic → Date, but the data model must not require a parent.
- Deleting an org or topic unassigns meetings; it does not delete them.

**First-slice difference:** the tree is required, and delete cascades (org → topics → groups → artifacts). Unassign-instead-of-delete is the later model.

### 6.4 Workspace UX

**First slice (shipped):** three panes — **Library** (strict tree, collapsible rail) · **Transcript** (meeting-group Files table + Transcript/playback cards, collapsible rail) · **Chat** (first-class pane). Settings is a header dialog. Dark charcoal / mint glass. No tasks rail, notes tab, or summary tab.

Preserve that three-region mental model as the product grows, simplified from BellaNote2:

1. **Library** — eventually meetings by date, with optional org/topic chips and filters. New meeting at the top. Settings reachable but not loud.
2. **Workspace** — title, tags, capture/ingest controls, waveform/playback, tabs: **Notes · Transcript · Summary · Chat**.
3. **Tasks** — persistent rail, collapsed to a thin tab. Mix of manual and confirmed-AI tasks. Link back to the source meeting.

**During recording:** large stop control, duration, input label, quiet live transcript. No modal AI.

**Empty states:** a new meeting should feel inviting (“Record, paste a YouTube link, or drop a file”) rather than like a blank IDE. First slice already teaches the tree: organization → topic → meeting group → files.

**Design bar:** ship the mint-pulse visual system. Motion is restrained. Density is desktop-class, not a blown-up phone web app. First-slice window is 1320×860 (min 1024×680).

### 6.5 AI tools

All AI reads **local text** (and later, related meeting text). Audio is not uploaded for MVP AI.

**Summary (A1)**  
Input: transcript, optional user notes, meeting type, optional audience picker (`executive`, `team`, `participants`, `absent stakeholder`).  
Output: markdown the user can edit and own.  
Templates at minimum:

- General conversation
- One-to-one
- Planning session
- Town hall / all-hands

Each template changes emphasis (commitments vs. decisions vs. narrative vs. Q&A themes) rather than adding fluff.

**Enhance notes (A2)**  
Only if notes exist. Rewrite *from the user’s jots*, grounded in the transcript. Keep provenance visually (user vs. model), as Granola does.

**Tasks (A3)**  
Model returns structured proposals: `{text, owner?, due?, evidence_timestamp?}`. User accepts, edits, or discards. Accepted items land in the tasks rail and remain editable.

**Chat (A4, later A5)**  
**First slice:** scoped RAG over Ready transcripts at org / topic / meeting group / this file. Newest groups first, ~110k character budget, omitted-count preview. The model is told to cite a discrete timestamp and artifact title. Click-cite-to-seek is not built. Refusal when the transcript does not contain the answer.  
When A5 is finished to spec: expandable file list, clickable citations that open the artifact and seek, and (later) a date-window retrieve that cites *which meeting* an answer came from.

**Guardrails**

- Customizable system prompts remain (power users loved this in the POC) but sit behind Settings, with reset-to-default.
- Quality filter: do not send garbage segments to the model by default.
- No training on user data by BellaNote. Document that the user’s chosen LLM vendor has its own policy.

### 6.6 Differentiating AI (still in-scope to *plan*, not all to *build*)

These are the “other powerful tools” from the brief, ranked by customer value vs. complexity:

| Idea | Why it matters | When |
|---|---|---|
| Audience-aware summaries | Same meeting, different readers | MVP |
| Confirmed task extraction | Turns talk into work | MVP |
| Related-meeting chat | The September onboarding question | P1 (scope chips shipped in first slice) |
| Decision log | Stops “wait, did we decide that?” | P1 |
| Follow-up email | Closes the meeting | P1 |
| Transcript confidence + re-transcribe | Trust in the source of truth | MVP / P1 |
| Multi-contributor merge | Team fidelity, unique in-category | P2 |
| “What changed since last time” | Recurring 1:1s and client accounts | P2–P3 |
| Handwriting + doc grounding | Full meeting memory | P3 |

Do **not** build live objection-coaching, talk-time scorecards, or “what to say next.” Those products exist (Gong, Convo). They fight the presence thesis.

### 6.7 Platform and engineering constraints

- **Performance:** Whisper stays in a long-lived worker. UI must stay responsive while transcribing. Library virtualizes long lists.
- **Efficiency:** One Rust host, small WebView UI, no second Electron-sized runtime if we can help it. Bundle size will be dominated by the Python/whisper payload — treat that as a known cost and keep it out of the hot UI path.
- **Windows parity:** Same product on Mac and Windows. System + mic uses ScreenCaptureKit on macOS and WASAPI loopback on Windows; the UI is not feature-flagged.
- **Security:** LLM keys in OS keychain (already). No secrets in the repo. Path handling for import/export stays audited (see existing `SECURITY.md` concerns).
- **Signing:** GitHub `.dmg` releases are signed and notarized with Developer ID Application. Setup is in `documentation/APPLE_CODE_SIGNING.md`. Windows NSIS installers ship from the same Release workflow.

### 6.8 Nice-to-have backend (P2)

The backend is **not** the recorder.

**Always local**

- Microphone and system capture
- Transcription
- Audio files (default)
- Ability to use the app with no account

**Backend is for**

- Identity and workspaces
- Uploading *transcripts and notes* (not audio, unless the user opts in)
- Sharing a meeting to people who also use BellaNote
- Receiving another participant’s transcript of the same meeting
- A reconcile view: side-by-side or merged transcript with conflict highlighting, to improve fidelity
- Shared org/topic vocabularies and shared task visibility
- Later: comment threads, version history

**Design implication now:** give every local meeting a stable id and a “remote id / share state” that can stay null. Do not postpone that column.

### 6.9 Long-term: meeting management (P3)

The product may grow from “beautiful notes from audio” into **the place a meeting lives**:

- Digitize handwritten pages (Rocketbook-class capture: photo → OCR → attach)
- Attach the deck, agenda, or pre-read
- Chat and summaries that infer across audio, handwriting, and documents
- Recurring meeting threads (“this is the same weekly planning”)

Requirements work for P3 should not distort MVP schema beyond: **a meeting can have many artifacts; a transcript is one artifact.**

### 6.10 Explicitly out of scope (all horizons unless we reopen)

- BellaNote-branded meeting bot
- Storing or streaming raw audio to BellaNote servers by default
- Video conference hosting
- Using customer audio to train BellaNote models
- Sales conversation intelligence (Gong-class)
- Auto-join every calendar event
- Cryptocurrency, marketplace, or social feed features

---

## 7. Suggested delivery sequence

### Phase 0 — Foundations (this repo)

- Product scope (this document) agreed
- **Done in the first slice:** app shell, design tokens, local schema for org / topic / meeting group / artifact, Mac file ingest, scoped chat
- Shared capture interface with macOS (ScreenCaptureKit) and Windows (WASAPI) backends — **shipped**

### Phase 1 — MVP desktop

- Remaining P0 rows in the feature table (summaries, tasks, attendees, export, consent)
- A person can go from zero to “I understand yesterday’s planning session” on both platforms without an account

### Phase 2 — Memory

- Search, related-meeting chat, calendar assist, speaker labels, recipes, follow-up email

### Phase 3 — Together

- Accounts, share, multi-contributor transcripts, shared tasks

### Phase 4 — Meeting workspace

- Handwriting, documents, richer inference, possible mobile capture

---

## 8. Risks and open questions

| Risk / question | Why it matters | Lean |
|---|---|---|
| Windows system-audio capture | Virtual-meeting story on Windows | **Shipped:** WASAPI loopback of the default render device, mixed with cpal mic |
| YouTube ToS / copyright | Ingest can look like a downloader | Ship as “import audio from a URL you are allowed to use,” block non-audio abuse, keep logs out of our servers |
| Whisper quality vs. cloud STT | Local models lag diarized cloud APIs | Keep audio so we can re-transcribe; quality tiers set expectations |
| BYO LLM keys vs. BellaNote-hosted AI | Keys are private but a setup step | MVP: BYO. Later: optional BellaNote-hosted for less technical users |
| Granola-like “you should type” temptation | Conflicts with the engagement thesis | Enhance-notes is optional. Empty notes still produce a full summary |
| Consent law varies by region | Bot-free is not consent-free | In-app reminder only in MVP; Jamie-style attendee email is P2 |
| Multi-contributor merge quality | Hard NLP problem | Start with compare + manual merge, not magic auto-merge |
| How opinionated is the visual redesign vs. the POC? | Speed vs. “beautiful” | Reuse the POC visual system unless we deliberately restyle |

---

## 9. Success metrics (MVP)

Qualitative first. We will not have SaaS dashboards on day one.

- Time from install to first completed transcript: under 10 minutes, including permissions
- A meeting captured with **no user notes** still produces a summary the founder would send to a colleague
- System+mic capture works on a Zoom or Teams call on **both** Mac and Windows without a bot
- A user can file the same meeting under an org, a topic, both, or neither, and find it again by date
- Chat answers a factual “who / what / when” question with a timestamp the user can click
- The app remains usable offline for record + transcribe

---

## 10. How to use this document next

The first Mac + Windows files-and-capture slice is in the repo. Story-level leftovers are listed at the bottom of [`USER_STORIES.md`](./USER_STORIES.md).

1. Treat remaining Section 5 P0 rows (summaries, tasks, attendees, export, consent) as the next requirements backlog.
2. Keep the first-slice schema (meeting group + artifacts) — a recording is another artifact in a group.
3. Port remaining BellaNote2 modules only where this slice did not already replace them: enhance-notes, quality filter, archive.

BellaNote succeeds if the note is beautiful *and* the person who made it was actually in the meeting.
