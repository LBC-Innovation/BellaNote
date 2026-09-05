# BellaNote — Business Logic, Value Scope & Product Requirements

**Document status:** Working draft for platform requirements  
**Audience:** Founder, product, and engineering  
**Date:** 5 September 2026  
**Repo:** [LBC-Innovation/BellaNote](https://github.com/LBC-Innovation/BellaNote)  
**Inputs:** Founder brief, existing BellaNote2 POC (`~/Documents/_DEV/_PERSONAL/BellaNote2`), [Granola](https://www.granola.ai/), and the 2026 AI meeting-notes category

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
| Capture | `voice` (mic via cpal) and `system` (ScreenCaptureKit mix of display audio + mic) | Keep both modes. Add a Windows WASAPI / loopback equivalent for `system`. |
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

- macOS only (no Windows capture path)
- No YouTube ingest, no local audio-file ingest
- No attendees, purpose, organization, or topic category
- Discovery chat is single-meeting only
- Tasks are not derived from the conversation
- Summaries are one generic executive voice, not audience/meeting-type aware
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
| C1 | Microphone recording | Capture a voice memo / in-person conversation from the device mic | MVP | P0 | Brief, POC | Existing `voice` mode. |
| C2 | System + microphone capture | Capture meeting playback *and* the user’s voice, no bot | MVP | P0 | Brief, POC, Granola | macOS: ScreenCaptureKit. Windows: WASAPI loopback + mic mix. Hardest Windows work. |
| C3 | YouTube URL ingest | User pastes a YouTube link; app fetches audio and transcribes locally | MVP | P0 | Brief | Personal-use helper. Must show ToS/copyright notice. Fail gracefully on restricted videos. |
| C4 | Local audio file ingest | User picks wav/mp3/m4a/ogg/etc.; transcribe locally | MVP | P0 | Brief, Market | Missing from POC. Reuse the whisper worker. |
| C5 | Import third-party transcript | Ingest VTT/SRT/TXT/DOCX from Teams, Zoom, Otter, etc.; audio optional | MVP | P0 | Brief, POC | POC has VTT only. Trust the file; do not require audio. |
| C6 | Re-transcribe | Re-run local model on kept audio (better model, or after edit) | Near-term | P1 | POC | POC already has regenerate. Keep it. |
| C7 | Live transcript (ignorable) | Segments appear while recording; user is not required to watch | MVP | P0 | POC, Otter | Calm, low-contrast during capture. |
| C8 | Local audio retention + playback | Keep audio on disk; waveform; click transcript to seek | MVP | P0 | POC, Diff | Opposite of Granola’s delete-audio default. Archive may drop audio (POC behavior). |
| C9 | Transcript quality display | Show low-confidence segments; optional filter before LLM use | MVP | P0 | POC, Diff | Already a technical edge. Surface it simply. |
| M1 | Meeting title | Editable name; default from time or first words | MVP | P0 | Brief, POC | POC has inline rename. |
| M2 | Attendees | Free-form people list; optional later contact pick | MVP | P0 | Brief | Not in POC. No directory required in MVP. |
| M3 | Meeting purpose | Short “why we met” field | MVP | P0 | Brief | Manual; AI may suggest after transcript exists. |
| M4 | Meeting type | Town hall, planning, 1:1, general, custom | MVP | P0 | Brief, Granola | Drives summary template. |
| M5 | Date and time | Always set; defaults to capture start; user-editable | MVP | P0 | Brief | Required. Used for library grouping. |
| M6 | Organization (optional) | User-defined org / client / company; not required | MVP | P0 | Brief | Independent of topic. |
| M7 | Topic category (optional) | User-defined topic; not required; independent of org | MVP | P0 | Brief | Same meeting may have org, topic, both, or neither. |
| M8 | Library by date | Today / yesterday / older, as in the POC | MVP | P0 | POC | Default view. |
| M9 | Filter / browse by org or topic | Narrow the library without forcing a tree | MVP | P0 | Brief | Hierarchy is a *view*, not a prison. Empty org/topic is valid. |
| M10 | Archive | Remove from active list; optionally drop audio | MVP | P0 | POC | Keep. |
| M11 | Full-text search | Search titles, transcript, notes, attendees | Near-term | P1 | Market | Users will ask for this immediately after MVP. |
| A1 | Audience-aware summary | Summary shaped by meeting type and chosen audience | MVP | P0 | Brief, POC | Evolve the existing executive-summary prompt into templates. |
| A2 | Enhance optional notes | If the user jotted anything, expand it from the transcript | MVP | P0 | POC, Granola | Keep the “your words vs AI words” distinction. |
| A3 | Assigned-task extraction | Propose owner + task + due date when spoken; user confirms | MVP | P0 | Brief, Market | Write into the tasks rail. Never silent-create. |
| A4 | Chat with this meeting | Request/response Q&A grounded in the transcript, with time cites | MVP | P0 | Brief, POC | Existing Discovery chat. Click cite → seek. |
| A5 | Chat across related meetings | If org and/or topic is set, retrieve sibling meetings as extra context | Near-term | P1 | Brief, Granola | The example in the brief (“new customer onboarding in September”) needs this. |
| A6 | Follow-up email draft | One-click draft recap + asks, copy or mailto | Near-term | P1 | Granola, Market | High perceived value, low build cost after A1. |
| A7 | Decision log | Extract “we decided X” as a first-class list | Near-term | P1 | Diff, Market | Complements tasks. Town halls and planning sessions especially. |
| A8 | Custom / saved prompts (“Recipes”) | User-saved post-meeting actions (PRD, coaching note, standup recap) | Near-term | P1 | Granola, Tactiq | POC already has editable system prompts; productize as recipes. |
| A9 | User-owned LLM | BYO key, provider + model picker, keychain storage | MVP | P0 | POC, Diff | Privacy story: BellaNote does not broker the user’s AI account in MVP. |
| A10 | Offline-safe core | Record and transcribe offline; AI disabled with a clear explanation | MVP | P0 | POC | Keep the existing offline modal behavior. |
| X1 | Copy / export transcript and notes | Markdown, plain text, VTT | MVP | P0 | POC, Market | POC exports timestamped text. Add markdown notes. |
| X2 | Light / dark theme | Ship both; dark default | MVP | P0 | POC | Brand continuity. |
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

---

## 6. Product scope (for platform requirements)

This section is the contract for the first requirements and architecture pass.

### 6.1 Product shape

- **Form factor:** Native desktop app for **macOS 13+** and **Windows 11**.
- **Stack direction (recommended):** Continue **Tauri 2 + Rust + React + TypeScript**. The POC already paid the tax on capture, bundling Whisper, keychain, and a dense desktop UI. Electron would fight the “efficient on both platforms” goal.
- **Local data:** SQLite + audio files in a user-chosen recordings directory.
- **AI:** Optional, user-configured. No AI vendor account required to record or transcribe.
- **Network:** Required only for LLM calls, YouTube ingest, and (later) sharing.

### 6.2 Capture and ingest

Every meeting is created the same way: **New meeting** (or “drop something on a meeting”). Then the user chooses an ingest path.

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
- File ingest: common containers (wav, mp3, m4a, aac, ogg, flac, webm). Reject video-as-product; if a video file is dropped, extract audio only.
- Transcript import: VTT, SRT, plain text, and at least one Teams/Zoom export shape. Preserve timestamps when present. `has_audio = false`.
- Stop recording is instant; trailing whisper chunks may still arrive (POC already handles this).
- User can rename, retag, and run AI after any ingest path. AI features do not care how the transcript arrived.

**Non-goals for capture**

- Video recording or gallery view
- A BellaNote bot that joins Zoom/Meet/Teams
- Automatic record-every-calendar-event (dangerous and consent-hostile)

### 6.3 Meeting model

A meeting is the atomic object.

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

**Hierarchy rules (founder constraint)**

- Date/time is always present.
- Organization and topic are **independent**. Valid states: neither, org only, topic only, both.
- The UI may *display* as Org → Topic → Date, but the data model must not require a parent.
- Deleting an org or topic unassigns meetings; it does not delete them.

### 6.4 Workspace UX

Preserve the POC’s three-region mental model, simplified:

1. **Library** — meetings by date, with optional org/topic chips and filters. New meeting at the top. Settings reachable but not loud.
2. **Workspace** — title, tags, capture/ingest controls, waveform/playback, tabs: **Notes · Transcript · Summary · Chat**.
3. **Tasks** — persistent rail, collapsed to a thin tab. Mix of manual and confirmed-AI tasks. Link back to the source meeting.

**During recording:** large stop control, duration, input label, quiet live transcript. No modal AI.

**Empty states:** a new meeting should feel inviting (“Record, paste a YouTube link, or drop a file”) rather than like a blank IDE.

**Design bar:** ship the mint-pulse visual system. Motion is restrained (the POC’s fade/shimmer language). Density is desktop-class, not a blown-up phone web app. Minimum window ~1280×800, as today.

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
Single-meeting RAG over the transcript with timestamp citations that seek the audio when present. Refusal when the transcript does not contain the answer.  
When A5 lands: retrieve top related meetings by org and/or topic and date window; cite *which meeting* an answer came from.

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
| Related-meeting chat | The September onboarding question | P1 |
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
- **Windows parity:** Feature-flag nothing in the UI. If system-audio capture slips, ship mic + file + import + YouTube on Windows rather than a “Mac only” badge on the main path — but **system+mic on Windows is still P0**, because that is the virtual-meeting story.
- **Security:** LLM keys in OS keychain (already). No secrets in the repo. Path handling for import/export stays audited (see existing `SECURITY.md` concerns).
- **Signing:** Plan for Apple notarization and Windows Authenticode before any external distribution. The POC already documents Gatekeeper “damaged app” failure mode.

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
- App shell, design tokens, local schema including optional org/topic
- Shared capture interface with macOS and Windows backends

### Phase 1 — MVP desktop

- All P0 rows in the feature table
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
| Windows system-audio capture is the schedule risk | This is the virtual-meeting story | Spike WASAPI loopback in Phase 0; do not discover it after UI is done |
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

1. Turn Section 5 (P0 rows) into the first requirements backlog / work items.
2. Write a short **platform architecture** note: capture trait, transcription worker, SQLite schema, LLM boundary.
3. Spike **Windows system audio** before investing in net-new UI.
4. Port, do not clone-and-forget, the POC modules that already work: capture, whisper worker, quality filter, enhance-notes, Discovery chat, theme.

BellaNote succeeds if the note is beautiful *and* the person who made it was actually in the meeting.
