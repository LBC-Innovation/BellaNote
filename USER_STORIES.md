# BellaNote — First-slice user stories

**Document status:** First Mac + Windows slice implemented — stories rewritten to the shipped app, not the original draft  
**Last review:** 6 September 2026 (Windows parity: WASAPI system audio + NSIS installer)  
**Platform:** macOS (Apple Silicon) and Windows desktop app (Tauri 2); system audio uses ScreenCaptureKit on Mac and WASAPI loopback on Windows  
**Slice goal:** A student (or anyone with a similar hierarchy) can create an organization and topic categories, file audio and existing transcripts into meeting groups, record from the microphone or system + mic, and ask natural-language questions against a chosen scope.

This slice is narrower than the full [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md). Summaries, task extraction, and a sharing backend are **out**. The Duke example below is the north-star walkthrough.

---

## How to read this

Each story uses:

- **Status** — `Complete` (shipped as described), `Partial` (shipped with leftover gaps), or `Not started`  
- **As a / I want / so that** — the user-facing intent  
- **Acceptance** — what the current app does  
- **Scenarios** — Given / When / Then, rewritten to match implementation  
- **Still open** — only on Partial stories; leftover work

Stories that shipped differently from the original draft were updated to the implemented behavior, not left as the older spec.

---

## North-star walkthrough (Duke)

1. Launch BellaNote on a Mac or Windows PC.  
2. Create organization **Duke**.  
3. Under Duke, create topic category **Competitive Strategies**.  
4. Inside that topic, create meeting groups **Lecture Class 1**, **Lecture Class 2**, **Mid Term Study Session**.  
5. In Lecture Class 1, add:
   - an audio file the professor posted after class  
   - a recording the student made on their phone  
   - a `.vtt` / `.txt` transcript downloaded from Zoom  
6. Ask: *“What did we say about switching costs in September?”* with scope set to **this topic**, or narrow it to **Lecture Class 1**, or to **one transcript**.

```
Duke                          ← Organization
└── Competitive Strategies    ← Topic category
    ├── Lecture Class 1       ← Meeting group
    │   ├── lecture-1.m4a     ← Audio → local transcript
    │   ├── my-phone.wav      ← Audio → local transcript
    │   └── zoom-export.vtt   ← Imported transcript (no audio required)
    ├── Lecture Class 2
    └── Mid Term Study Session
```

---

## Domain terms for this slice

| Term | Meaning | Example |
|---|---|---|
| **Organization** | Top-level container (school, company, client) | Duke |
| **Topic category** | A subject or course inside an organization | Competitive Strategies |
| **Meeting group** | A named bundle of related sessions / artifacts | Lecture Class 1 |
| **Artifact** | One audio file or one imported transcript living in a meeting group | `zoom-export.vtt` |
| **Transcript** | The text BellaNote will chat over — produced locally from audio, or imported | |
| **Chat scope** | How wide the model is allowed to look | Org / topic / meeting group / this file |

**Refinement vs. product scope:** [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md) still describes the long-term optional-tag meeting. This slice **ships** meeting group as a first-class folder of artifacts on the path **Org → Topic → Group → Artifact**. Unfiled org/topic items are out of this slice.

---

## Slice in / out

| In (shipped unless noted) | Out |
|---|---|
| Launch on Mac and Windows | Audience-aware summaries, task extraction |
| Create / rename / delete org, topic, meeting group | YouTube URL ingest |
| Record microphone (Mac and Windows) | Sharing / multi-user backend |
| Record system + mic on macOS (ScreenCaptureKit audio + cpal mic) and Windows (WASAPI loopback + cpal mic) | Calendar, speaker diarization |
| Upload one or more audio files → one-shot `small.en` on device; queue extras | |
| Import one or more `.vtt` / `.srt` / `.txt` | Mobile |
| View artifacts in the hierarchy; remove one or many with confirm | Move an artifact between groups (US-205, not started) |
| Export a meeting’s audio to a folder I choose (library copy stays) | |
| Static waveform + click-to-seek; play / Follow / speed / clock under the waveform; search filters transcript lines | |
| ChatGPT (`gpt-4o`) with an explicit scope | |
| User-provided OpenAI API token in the keychain | |

---

## Story map

```
Epic 1  Launch & hierarchy
        ~~US-101  Launch the Mac / Windows app~~            Complete
        ~~US-102  Create an organization~~                  Complete
        ~~US-103  Create a topic category~~                 Complete
        ~~US-104  Create a meeting group~~                  Complete
        ~~US-105  Browse the hierarchy~~                    Complete
        ~~US-106  Rename containers~~                       Complete
        ~~US-107  Delete containers~~                       Complete

Epic 2  Files & transcripts
        ~~US-201  Add an audio file to a meeting group~~    Complete
        ~~US-202  Watch transcription finish~~              Complete
        ~~US-203  Import an existing transcript~~           Complete
        ~~US-204  Open and read an artifact~~               Complete
        US-205  Re-file / move an artifact                 Not started
        ~~US-206  Remove an artifact~~                      Complete
        ~~US-207  Search within a transcript~~              Complete
        ~~US-209  Record from the microphone~~              Complete
        ~~US-210  Record system audio + microphone~~        Complete
        ~~US-211  Export a meeting audio file~~             Complete
        US-208  Leave a timed comment on audio             Not started

Epic 3  Scoped chat
        ~~US-301  Save the ChatGPT API token~~              Complete
        ~~US-302  Ask a question against a chosen scope~~   Complete
        US-303  Read a well-organized answer               Partial
        US-304  Continue a conversation                    Partial
        US-305  See what the model used (and did not use)  Partial
```

---

# Epic 1 — Launch the app and build the hierarchy

## ~~US-101 — Launch BellaNote on a Mac or Windows PC~~

**Status:** Complete

**As a** student on a Mac or Windows PC,  
**I want** to open BellaNote like any other desktop app,  
**so that** I can start organizing class material without a browser or account.

**Acceptance**

- A standard Tauri app launches to a local three-pane workspace (Library, Transcript, Chat).  
- The titlebar has layout buttons (Library only / Transcript only / Chat only) and Settings. The empty middle strip is the window drag region. On macOS, a left inset clears the overlay traffic lights. Layout buttons sit on the right edge on Mac and Windows.  
- No sign-in is required.  
- First launch shows an empty library (“Start with an organization”) and a workspace prompt to create an organization.  
- Closing and reopening restores organizations, topics, groups, and artifacts from local SQLite.  
- Library, transcript, and chat collapse-to-rail state is remembered; the last selected tree row is not (selection starts unset).

### Scenarios

**Happy path — first launch**

- **Given** BellaNote is installed and has never been opened  
- **When** I launch it  
- **Then** I see the empty library and a primary **Organization** action  
- **And** the workspace explains the Duke-style tree  
- **And** I am not asked for an account or cloud login

**Relaunch restores work**

- **Given** I previously created Duke / Competitive Strategies / Lecture Class 1  
- **When** I quit and launch again  
- **Then** that hierarchy is still in the library  
- **And** no tree row is pre-selected  
- **And** collapsed library / transcript / chat rails stay as I left them

**Already running**

- **Given** BellaNote is already open  
- **When** I launch it again  
- **Then** macOS or Windows brings the existing window forward (one app window)

**Permissions (this slice)**

- **Given** I have not yet added audio  
- **When** the app launches  
- **Then** it does not demand Microphone or Screen Recording permission  
- **And** it only asks for file access when I pick a file

---

## ~~US-102 — Create an organization~~

**Status:** Complete

**As a** student,  
**I want** to create an organization named after my school,  
**so that** all of my Duke material lives in one place.

**Acceptance**

- I can create an organization from the library **Organization** button (or the collapsed-rail plus).  
- The new org appears immediately as a top-level library row and becomes the selection.  
- Names are trimmed; empty / whitespace-only names are rejected with “A name is required.”  
- Names are capped at 80 characters.  
- Duplicate org names are blocked case-insensitively (“An organization with that name already exists.”).  
- I can have more than one organization (e.g. Duke and a summer internship).

### Scenarios

**Happy path**

- **Given** I am in the empty or existing library  
- **When** I create an organization named `Duke`  
- **Then** Duke appears as a top-level item  
- **And** the workspace asks me to add a topic under Duke

**Whitespace-only name**

- **Given** the create-organization field is open  
- **When** I enter `   ` and confirm  
- **Then** the org is not created  
- **And** I see “A name is required.”

**Duplicate name**

- **Given** Duke already exists  
- **When** I try to create another organization named `Duke` (or `duke`)  
- **Then** the app blocks it and shows that an organization with that name already exists

**Very long name**

- **Given** I paste a 300-character name  
- **When** I type in the field  
- **Then** input stops at 80 characters  
- **And** the library row truncates instead of breaking the layout

---

## ~~US-103 — Create a topic category~~

**Status:** Complete

**As a** Duke student,  
**I want** to add a topic category for a class,  
**so that** Competitive Strategies is distinct from my other courses.

**Acceptance**

- From an organization row I can create a topic with a name (inline **Topic** control).  
- The topic appears nested under that organization and becomes the selection.  
- A topic is not required to contain meeting groups yet.  
- Duplicate topic names in the same org are blocked case-insensitively.  
- The same topic name may exist under a different organization.  
- There is no way to create a topic without an organization.

### Scenarios

**Happy path**

- **Given** organization Duke exists  
- **When** I create a topic category `Competitive Strategies` under Duke  
- **Then** it appears under Duke  
- **And** the workspace asks me to add a meeting group  
- **And** a **Meeting group** action sits under that topic in the library

**Second course**

- **Given** Duke already has Competitive Strategies  
- **When** I add `Corporate Finance` under Duke  
- **Then** both topics appear under Duke  
- **And** they do not share meeting groups

**Topic with the same name in a different org**

- **Given** Duke has Competitive Strategies  
- **And** I also have an organization `Coursera`  
- **When** I create Competitive Strategies under Coursera  
- **Then** both topics exist independently

**Duplicate topic in the same org**

- **Given** Duke already has Competitive Strategies  
- **When** I create another topic named `Competitive Strategies` under Duke  
- **Then** the app blocks it (“That topic already exists in this organization.”)

**Topic without an organization**

- **Given** I have not created any organization  
- **When** I look for a way to create a topic  
- **Then** I must create an organization first  
- **And** the empty states say so

---

## ~~US-104 — Create a meeting group~~

**Status:** Complete

**As a** student in Competitive Strategies,  
**I want** to create named meeting groups for each lecture or study session,  
**so that** several recordings and transcripts from the same class meeting live together.

**Acceptance**

- From a topic I can create a meeting group with a name (inline **Meeting group** control).  
- The group is dated at creation (now) and that date is shown as secondary text on the library row.  
- The date is not editable in this slice.  
- Duplicate names in the same topic are allowed; rows are distinguished by date.  
- The group can be empty. Opening it shows the Files card and an **Import Meeting** control (**Audio files** / **Transcript files**), which opens a multi-file drop-zone modal.  
- A meeting group cannot be created except under a topic.

### Scenarios

**Happy path**

- **Given** I am in Duke → Competitive Strategies  
- **When** I create meeting groups `Lecture Class 1`, `Lecture Class 2`, and `Mid Term Study Session`  
- **Then** all three appear under the topic  
- **And** each can be opened and shows an empty file list plus **Import Meeting**

**Default date**

- **Given** today is 5 September 2026  
- **When** I create Lecture Class 1  
- **Then** the group is dated today  
- **And** that date appears under the group name in the library

**Duplicate group name in the same topic**

- **Given** Lecture Class 1 already exists in Competitive Strategies  
- **When** I create another `Lecture Class 1` in the same topic  
- **Then** both groups exist  
- **And** the library distinguishes them by date

**Group without a topic**

- **Given** I am looking at Duke with no topic selected  
- **When** I try to create a meeting group  
- **Then** I must pick or create a topic first

---

## ~~US-105 — Browse the hierarchy~~

**Status:** Complete

**As a** student,  
**I want** to move from Duke down to a single lecture group in a few clicks,  
**so that** I always know where I am and what files belong together.

**Acceptance**

- Library is a tree: Org → Topic → Meeting group. Organizations and topics collapse; clicking anywhere on the row (except the overflow menu) expands or collapses that branch.  
- **Collapse all** folds every org and topic. Expanded / collapsed branch ids persist (`bellanote.libraryTreeCollapsed`).  
- Selecting a row highlights it and updates the workspace; siblings stay visible when their parent is expanded.  
- The workspace header breadcrumb shows the path (e.g. Duke / Competitive Strategies / Lecture Class 1).  
- Selecting an org or topic shows a teaching empty state; selecting a group shows that group’s files.  
- Library, transcript, and chat panes each collapse to a vertical rail (click the rail to expand). Rail state persists.  
- Titlebar **Library only / Transcript only / Chat only** focuses that pane and collapses the other two to rails. Pane widths animate to the new layout instead of jumping.  
- Empty levels have a useful empty state, not a blank panel.

### Scenarios

**Drill in**

- **Given** the Duke example data exists  
- **When** I click Duke, then Competitive Strategies, then Lecture Class 1  
- **Then** the workspace shows only the artifacts in Lecture Class 1  
- **And** the breadcrumb reads Duke / Competitive Strategies / Lecture Class 1  
- **And** the rest of the tree remains visible in the library

**Org with many topics**

- **Given** Duke has 12 topic categories  
- **When** I open Duke  
- **Then** the library list is scrollable and readable  
- **And** I can find Competitive Strategies without renaming anything

**Empty topic**

- **Given** I just created Competitive Strategies  
- **When** I open it  
- **Then** the workspace tells me to add a meeting group  
- **And** I do not see other topics’ files

**Collapse a branch**

- **Given** Duke is expanded and shows Competitive Strategies  
- **When** I click the Duke row (not the menu)  
- **Then** its topics hide  
- **And** that collapsed state is still there after I quit and reopen

**Collapse a pane**

- **Given** I am reading a transcript and chatting  
- **When** I collapse Library, Transcript, or Chat  
- **Then** that pane becomes a labeled rail  
- **And** the remaining panes take the leftover space  
- **And** the widths ease into place instead of snapping

**Focus one pane from the titlebar**

- **Given** all three panes are open  
- **When** I click **Show transcript only**  
- **Then** Library and Chat become rails  
- **And** Transcript fills the leftover space  
- **And** the transcript layout button stays pressed until I expand another pane

---

## ~~US-106 — Rename a container~~

**Status:** Complete

**As a** student,  
**I want** to rename an organization, topic, or meeting group,  
**so that** a typo or a better name does not force me to recreate the tree.

**Acceptance**

- Rename from the row overflow menu (Rename), via a dialog with the same 80-character / trim rules as create.  
- Children stay attached after rename.  
- Same uniqueness rules as create (org and topic blocked; meeting groups may collide).  
- Chat scopes keep working because they use stable ids, not names.

### Scenarios

**Rename topic**

- **Given** the topic is named `Comp Strat`  
- **When** I rename it to `Competitive Strategies`  
- **Then** all meeting groups and artifacts remain under it  
- **And** chat scopes that pointed at this topic still resolve to it

**Rename to a duplicate**

- **Given** Duke has Competitive Strategies and Corporate Finance  
- **When** I rename Corporate Finance to Competitive Strategies  
- **Then** the same uniqueness rule as US-103 applies

---

## ~~US-107 — Delete a container~~

**Status:** Complete

**As a** student,  
**I want** to delete an organization, topic, or meeting group I no longer need,  
**so that** my library stays tidy.

**Acceptance**

- Delete from the row overflow menu always asks for confirmation (empty or not).  
- Copy explains that the item and everything inside it leaves BellaNote, and that original files on disk are not deleted.  
- Confirm cascades: org → topics → groups → artifacts (and BellaNote’s library copies).  
- Cancel leaves everything unchanged.  
- There is no undo.

### Scenarios

**Delete empty group**

- **Given** Lecture Class 1 has no artifacts  
- **When** I delete it and confirm  
- **Then** it disappears from Competitive Strategies

**Delete group with files**

- **Given** Lecture Class 1 contains two audio transcripts and one imported VTT  
- **When** I choose delete  
- **Then** I must confirm  
- **And** those artifacts are removed from BellaNote  
- **And** the original files on disk are not deleted — only BellaNote’s copies

**Delete a topic**

- **Given** Competitive Strategies has three meeting groups  
- **When** I delete the topic and confirm  
- **Then** the topic and its groups and artifacts are gone  
- **And** Duke remains

**Delete an organization**

- **Given** Duke has topics and groups  
- **When** I delete Duke and confirm  
- **Then** the entire tree under Duke is removed  
- **And** other organizations are untouched

**Undo**

- **Given** I just deleted a meeting group  
- **When** I look for undo  
- **Then** there is no undo; confirmation is the safety net

---

# Epic 2 — Add files and organize transcripts

## ~~US-201 — Add an audio file to a meeting group~~

**Status:** Complete

**As a** student,  
**I want** to choose an audio file from my Mac and attach it to a meeting group,  
**so that** a posted lecture recording becomes part of Lecture Class 1.

**Acceptance**

- From a meeting group, **Import Meeting → Audio files** opens a drop-zone modal (not an immediate single-file picker).  
- Click the drop canvas to pick one or more audio files (`wav`, `mp3`, `m4a`, `aac`, `ogg`, `flac`). Dropping files onto the open modal / window also adds them.  
- Each file is copied into Application Support (`library/{id}/source.{ext}`) and imported in sequence so extras queue.  
- The artifact title defaults to the file stem (editable later); the original filename is shown as secondary text.  
- Transcription starts automatically (`small.en` one-shot). If another job already holds the worker, later files stay **Pending**.  
- Unsupported types and non-MP4 video (`mov` / `mkv` / etc.) are rejected with a readable reason. Use **Import Meeting → Video files** for MP4.  
- Adding the same path again creates a second artifact (no duplicate warning).

### Scenarios

**Happy path — professor’s recording**

- **Given** I am in Duke / Competitive Strategies / Lecture Class 1  
- **When** I open **Import Meeting → Audio files** and choose `lecture-1.m4a`  
- **Then** the artifact appears in the Files table immediately  
- **And** its quality shows **Importing** once the worker starts (or **Pending** if another file is already importing)  
- **And** I can keep using the app

**Phone recording from class**

- **Given** I am in the same meeting group and `lecture-1.m4a` is already importing  
- **When** I add `voice-memo.wav`  
- **Then** it appears as a second artifact  
- **And** it shows **Pending** with an amber row until the first job finishes  
- **And** I cannot open the pending row in the transcript card

**Unsupported file**

- **Given** the audio picker only lists audio extensions  
- **When** a non-audio path is still submitted  
- **Then** the file is not added  
- **And** I see that BellaNote needs an audio file

**Video file on the audio path**

- **Given** I try to add `lecture.mp4` through **Audio files**  
- **When** I add it  
- **Then** BellaNote rejects it and points me to **Import Meeting → Video files**

**MP4 via Video files**

- **Given** I open **Import Meeting → Video files** and choose `lecture.mp4`  
- **When** BellaNote imports it  
- **Then** the artifact appears with a **Video** chip, audio is extracted for playback/transcription, and **Watch** opens the stored video

**Duplicate of the same file**

- **Given** `lecture-1.m4a` is already in Lecture Class 1  
- **When** I add the same path again  
- **Then** a second artifact is created (a new copy in the library)

**File disappears after pick**

- **Given** I selected a file on a USB drive  
- **When** the drive is ejected before the copy finishes  
- **Then** the add fails with a readable “Couldn’t read / copy the file” error  
- **And** if the copy already succeeded, transcription can continue from BellaNote’s copy

**Very large file**

- **Given** I add a 3-hour lecture  
- **When** transcription starts  
- **Then** the UI stays usable  
- **And** the row shows **Importing** with a spinner (no percent)

---

## ~~US-202 — See transcription complete (or fail) and retry~~

**Status:** Complete

**As a** student,  
**I want** to know when an audio file has become a transcript I can read and chat with,  
**so that** I am not guessing whether the file is ready.

**Acceptance**

- Internal status values: `queued`, `transcribing`, `ready`, `failed`.  
- Files table Quality column: **Pending** (queued), **Importing** + spinner (the job that holds the worker), **small** when ready, **Failed** + **Retry** when failed.  
- Only the currently transcribing file shows Importing. Other waiting uploads show Pending, use an amber row, and cannot be loaded in the transcript card.  
- Ready artifacts have transcript text and are eligible for chat. Audio with an empty transcript is **Failed**, not Ready.  
- Failed artifacts can be opened and show a short error plus Retry in the transcript card. Retry transcribes BellaNote’s saved copy.  
- Chat never includes artifacts that are not Ready.  
- One faster-whisper worker; extra jobs wait on a lock (usually in upload order, not a named FIFO queue).  
- Quit mid-import marks leftover `queued` / `transcribing` rows as Failed on next launch, with Retry.  
- Launch also marks leftover `ready` audio rows with an empty transcript as Failed, with Retry.  
- Clicking an **Importing** row does not load it in the transcript card. A top-center warning toast says **Please wait, this file is importing** (icon + warning colors), then fades.

### Scenarios

**Success**

- **Given** `lecture-1.m4a` was added  
- **When** local transcription finishes  
- **Then** Quality becomes **small**  
- **And** I can open the transcript  
- **And** the artifact is eligible for chat

**Failure — unreadable audio**

- **Given** the file is corrupt or unusable  
- **When** transcription fails  
- **Then** Quality is **Failed**  
- **And** a Retry button is on the row  
- **And** opening the row shows a human sentence, not a stack trace  
- **And** Retry queues the same file again

**Failure — empty transcript after a recording**

- **Given** the audio file was saved  
- **When** Whisper never returns text (worker crash, no speech, or a prior ready-with-empty row)  
- **Then** Quality is **Failed**  
- **And** the transcript card says the recording is saved and can be tried again  
- **And** Retry transcribes `source.wav` without recording again

**Quit mid-transcribe**

- **Given** transcription is in progress  
- **When** I quit the app  
- **Then** on next launch the row says Failed  
- **And** Retry starts the import again

**Two files at once**

- **Given** I added two audio files one after the other  
- **When** the first is importing  
- **Then** the second shows **Pending** and cannot be opened  
- **And** when the first finishes (ready or failed), the second becomes **Importing**  
- **And** one failure does not cancel the other

**Click while importing**

- **Given** a row shows **Importing**  
- **When** I click it  
- **Then** the transcript card stays on whatever was already open  
- **And** a warning toast tells me to wait

---

## ~~US-203 — Import an existing transcript file~~

**Status:** Complete

**As a** student,  
**I want** to add a transcript I already have (Zoom, Teams, or a plain text file) to a meeting group,  
**so that** I can organize it next to recordings without re-transcribing.

**Acceptance**

- From a meeting group, **Import Meeting → Transcript files** opens the same style of drop-zone modal as audio. Click or drop one or more `.vtt`, `.srt`, or `.txt` files.  
- Zoom / Teams cue files are parsed when they contain `-->` timestamps (typical VTT/SRT). There is no separate DOCX or proprietary export parser.  
- Audio is not required. Quality shows **Imported**.  
- Timestamps are kept when the file has them.  
- Empty files and files over 2 MB of text are rejected with a reason.

### Scenarios

**Happy path — Zoom download**

- **Given** I am in Lecture Class 1  
- **When** I import `zoom-export.vtt`  
- **Then** the artifact appears as **Imported** (no transcribe wait)  
- **And** I can read the cue text in order  
- **And** I can include it in chat

**Plain text notes**

- **Given** I have `study-session.txt`  
- **When** I import it  
- **Then** the full text is the transcript  
- **And** missing timestamps are OK

**Empty file**

- **Given** I pick a 0-byte `.txt`  
- **When** I import  
- **Then** the import is rejected  
- **And** I am told the file has no text

**Wrong extension, right content**

- **Given** a Zoom transcript saved as `.txt`  
- **When** I import it via **Transcript files**  
- **Then** if the file contains `-->` cues it is parsed as timed text  
- **And** otherwise it is treated as plain text

**Audio mistaken for transcript**

- **Given** I choose `lecture-1.m4a` in the transcript picker  
- **When** I confirm  
- **Then** BellaNote rejects it and says it looks like audio

**Huge text file**

- **Given** a 20 MB dump  
- **When** I import  
- **Then** the import is rejected (“That transcript is larger than 2 MB.”)

---

## ~~US-204 — Open an artifact and see its transcript~~

**Status:** Complete

**As a** student,  
**I want** to click an artifact and read the transcript in a readable layout,  
**so that** I can skim a lecture before I ask questions.

**Acceptance**

- The meeting-group workspace stacks two cards: **Files** and **Transcript**. Both can be open at once.  
- Files table columns: file title + original filename, date added, quality.  
- The Transcript card header shows chips for what that file actually has: **Audio** only if `has_audio`, **Transcript** only when status is Ready and text exists. An imported `.vtt` does not get an Audio chip.  
- Ready and failed rows can be selected (outlined card). Pending rows are amber and not loadable. Importing rows are not loadable; a click shows the wait toast (US-202).  
- Ready audio artifacts show a full-width static waveform and click-to-seek. A control row **under** the waveform has play/pause, **Follow**, a playback-speed menu (**1x / 1.25x / 1.5x / 2x**, remembered), and a playhead / total clock. Follow is its own button; the clock is display-only.  
- Imported transcripts have no waveform.  
- Long transcripts scroll in the transcript card. A **Search transcript** field under the lines filters them as I type (US-207).  
- Inline rename (pencil) and delete (trash) sit on each row. **Select multiple** (checkboxes on the right, same slot as edit/delete) appears only while the Files accordion is open.

### Scenarios

**Read imported VTT**

- **Given** `zoom-export.vtt` is Ready  
- **When** I open it  
- **Then** I see the text in reading order  
- **And** timestamps display if present  
- **And** Quality is **Imported**  
- **And** the Transcript card shows a **Transcript** chip only

**Open while still importing**

- **Given** `lecture-1.m4a` is Importing  
- **When** I click the row  
- **Then** the transcript card does not switch to it  
- **And** a warning toast says to wait

**Pending file is not loadable**

- **Given** a second audio file is Pending  
- **When** I click that row  
- **Then** the transcript card does not switch to it

**Play audio**

- **Given** a ready audio artifact is selected  
- **When** I press play or click the waveform  
- **Then** playback seeks to that point  
- **And** the clock under the waveform shows the current playhead and the total length  
- **And** the matching transcript line highlights if Follow is on

**Toggle Follow**

- **Given** a ready audio artifact is playing  
- **When** I click **Follow** on the control row  
- **Then** Follow turns off (or on again)  
- **And** transcript auto-scroll stops when Follow is off

**Change playback speed**

- **Given** a ready audio artifact is selected  
- **When** I open the speed menu and choose **1.5x**  
- **Then** playback runs at one-and-a-half speed  
- **And** the next file I open still starts at 1.5x

**Rename artifact**

- **Given** the title is `lecture-1`  
- **When** I rename it to `Lecture 1 — professor recording`  
- **Then** the Files table uses the new title  
- **And** chat uses the new title the next time that file is sent as context

**Select multiple is hidden when Files is collapsed**

- **Given** the Files accordion is open and I can see **Select multiple**  
- **When** I collapse Files  
- **Then** that button is gone  
- **And** multi-select mode is cleared

---

## US-205 — Move an artifact to another meeting group

**Status:** Not started

**As a** student,  
**I want** to move a file I dropped in the wrong group,  
**so that** organization stays accurate without re-importing.

**Acceptance** *(not built)*

- I can move an artifact to another meeting group (same topic or another topic/org).  
- Transcript and status travel with it.  
- Chat scopes follow the new location.

### Scenarios

**Wrong lecture**

- **Given** `zoom-export.vtt` is in Lecture Class 1  
- **When** I move it to Lecture Class 2  
- **Then** it no longer appears in Class 1  
- **And** it appears in Class 2 as Ready

**Move across topics**

- **Given** an artifact is in Competitive Strategies / Lecture Class 1  
- **When** I move it to Corporate Finance / Week 1  
- **Then** only Corporate Finance chat scopes include it afterward

---

## ~~US-206 — Remove an artifact~~

**Status:** Complete

**As a** student,  
**I want** to remove a file I added by mistake,  
**so that** it no longer appears in the group or in chat.

**Acceptance**

- Trash on the file row asks for confirmation.  
- **Select multiple** puts a checkbox on each row (including pending / importing / failed). **Delete** then confirms “Are you sure you want to delete {N} file(s)?”  
- Those controls are hidden when the Files accordion is collapsed.  
- Removing an **imported** file always deletes BellaNote’s library copy and the DB row. The user’s original file (Downloads or wherever it was picked) is **never** deleted.  
- Removing a **recording** asks optionally: “Also delete BellaNote’s copy of the audio” (off by default). Checking it removes the library WAV; leaving it unchecked keeps that file on disk under Application Support / AppData.  
- Removed artifacts are out of every chat scope on the next ask.  
- Removing a pending or importing file deletes the row immediately. A worker that had already started may finish in the background and then have nothing to update.

### Scenarios

**Remove imported transcript**

- **Given** `zoom-export.vtt` is in Lecture Class 1  
- **When** I remove it and confirm  
- **Then** the group no longer lists it  
- **And** a topic-scoped chat no longer uses that text

**Remove during import**

- **Given** an audio file is Importing or Pending  
- **When** I remove it and confirm  
- **Then** it is gone from the group  
- **And** a later worker result for that id is ignored because the row is gone

**Remove several files**

- **Given** Lecture Class 1 has three files  
- **When** I choose Select multiple, check two rows, and confirm Delete  
- **Then** those two are gone  
- **And** the third remains

---

## ~~US-207 — Search within a transcript~~

**Status:** Complete

**As a** student,  
**I want** to type into a search field and see only the transcript lines that match,  
**so that** I can jump to a phrase in a long lecture without scrolling the whole file.

**Acceptance**

- Ready transcripts show a **Search transcript** field under the scrollable lines (not over the waveform).  
- Typing filters lines immediately (case-insensitive substring on the line text).  
- A clear control restores the full list. Switching to another file clears the query.  
- When nothing matches, the list says so.  
- Follow does not auto-scroll the list while a query is active, so the filtered results stay put. Clicking a visible timestamped line still seeks the audio.

### Scenarios

**Filter as I type**

- **Given** Lecture 1’s transcript is open with several lines  
- **When** I type `strategy` in **Search transcript**  
- **Then** only lines that contain that word remain  
- **And** unmatched lines are hidden

**No matches**

- **Given** I am searching the same transcript  
- **When** I type a word that does not appear  
- **Then** I see that no lines match  
- **And** the search field still shows what I typed

**Clear search**

- **Given** the list is filtered  
- **When** I clear the search  
- **Then** every line is visible again

---

## ~~US-209 — Record from the microphone~~

**Status:** Complete

**As a** student in a live discussion,  
**I want** to record from this computer’s microphone into the current meeting group,  
**so that** I get an on-device transcript without importing a file afterwards.

**Acceptance**

- Next to **Import Meeting**, **Record Meeting** opens a dropdown: **Microphone** / **System audio**.  
- **Microphone** starts capture immediately on the default input (`cpal`). macOS asks for Microphone permission the first time.  
- One recording at a time. While it runs, Import Meeting is disabled and the control becomes **Stop · MM:SS**.  
- A new artifact appears in Files with status **Recording**. Chunks of transcript may appear while capturing. Stop is instant; BellaNote then transcribes the saved `source.wav` (status **Importing**) and must not mark Ready with an empty transcript.  
- The recording is stored as `library/{id}/source.wav` and plays like an imported audio file once Ready. Microphone-only is 16 kHz dual-mono; system + mic is 48 kHz stereo.  
- Closing BellaNote mid-record marks the row Failed.

### Scenarios

**Happy path**

- **Given** Lecture Class 1 is selected  
- **When** I choose **Record Meeting → Microphone** and speak, then Stop  
- **Then** a microphone recording artifact is in the Files list  
- **And** it becomes Ready with a transcript and waveform

**Already recording**

- **Given** a recording is in progress  
- **When** I try to start another  
- **Then** BellaNote keeps the first one and does not start a second

---

## ~~US-210 — Record system audio and the microphone~~

**Status:** Complete (macOS ScreenCaptureKit; Windows WASAPI loopback)

**As a** person in a virtual meeting,  
**I want** BellaNote to capture what I hear and what I say on one timeline,  
**so that** I do not need a virtual audio cable or a meeting bot.

**Acceptance**

- **Record Meeting → System audio** captures display/playback audio and the microphone via `cpal`, mixed with gain staging and a soft limiter.  
- macOS uses ScreenCaptureKit audio (not video) plus the microphone, and asks for Screen Recording and Microphone.  
- Windows uses WASAPI loopback of the default playback device (“what you hear”) plus the microphone. Headphones avoid speaker echo into the mic.  
- While recording, a short consent line is visible: you are capturing audio on this device.  
- Mixed audio is one 48 kHz stereo `source.wav` (system L/R preserved; mic centered). Whisper still transcribes a 16 kHz mono downmix.

### Scenarios

**Virtual meeting on a Mac**

- **Given** Lecture Class 1 is selected and Zoom is playing  
- **When** I choose **Record Meeting → System audio**, speak, then Stop  
- **Then** the artifact contains meeting playback mixed with my voice  
- **And** the transcript covers both

**Virtual meeting on Windows**

- **Given** I am on Windows with a playback device and Zoom or Teams is playing  
- **When** I choose **Record Meeting → System audio**, speak, then Stop  
- **Then** the artifact contains meeting playback mixed with my voice  
- **And** the transcript covers both

---

## ~~US-211 — Export a meeting audio file~~

**Status:** Complete

**As a** student,  
**I want** to export the audio BellaNote saved for a meeting,  
**so that** I can keep a copy outside the app without removing it from my library.

**Acceptance**

- Files that have audio show an **Export** control on the Files row and an **Export** button on the playback toolbar.  
- Choosing Export opens a save dialog; the default name uses the original filename when present, otherwise the file title and the library extension.  
- Confirming copies BellaNote’s library `source.*` to that path. The library file is not moved or deleted.  
- Canceling the dialog does nothing.  
- Transcript-only files do not offer Export.  
- While a recording is still in progress, Export is hidden.

### Scenarios

**Export a recording**

- **Given** a microphone or system recording is Ready (or Failed with audio on disk)  
- **When** I choose Export and pick a destination  
- **Then** a copy of that audio is at the destination  
- **And** the file still plays in BellaNote

**Cancel**

- **Given** I open the Export save dialog  
- **When** I cancel  
- **Then** nothing is written outside the library

---

## US-208 — Leave a timed comment on audio

**Status:** Not started

**As a** student,  
**I want** to pin a short comment to a moment in the audio,  
**so that** I can jump back to that part of the lecture from the waveform or a list.

**Acceptance**

- On a ready audio artifact, a comment button sits to the right of **Search transcript**.  
- Clicking it remembers the current playhead (or `00:00` if playback has not started) and expands a text field to the right of the button.  
- Saving stores the comment with that timecode. Empty text does not save. Escape or a second click on the button (while the field is empty) cancels.  
- Each saved comment draws a vertical line on the waveform with a small round node at the top. Clicking the node seeks and starts playback at that time.  
- A **Comments** accordion sits under the Transcript card. It lists Time and Comment, oldest timecode first. Clicking a row seeks and starts playback at that time.  
- Comments persist locally with the artifact (deleted with the file). Transcript-only files have no comment button, markers, or Comments card.  
- The comment field is capped at 500 characters.

### Scenarios

**Pin a moment while listening**

- **Given** `lecture-1.m4a` is Ready and playing at `12:04`  
- **When** I click the comment button next to Search transcript  
- **Then** a text field opens to the right of the button  
- **And** it is tied to `12:04`

**Save the comment**

- **Given** the comment field is open at `12:04`  
- **When** I type `switching costs example` and save  
- **Then** the field closes  
- **And** a marker appears on the waveform at `12:04`  
- **And** Comments lists `12:04` / `switching costs example`

**Jump from the waveform**

- **Given** a comment exists at `12:04`  
- **When** I click its node on the waveform  
- **Then** playback starts at `12:04`

**Jump from the table**

- **Given** Comments is open and lists two comments  
- **When** I click the row for `12:04`  
- **Then** playback starts at `12:04`

**Cancel without saving**

- **Given** the comment field is open and empty  
- **When** I press Escape  
- **Then** the field closes  
- **And** no comment or marker is added

**Transcript-only file**

- **Given** `zoom-export.vtt` is selected  
- **When** I look at the transcript card  
- **Then** there is no comment button, waveform marker, or Comments card

---

# Epic 3 — Chat with a chosen scope

## ~~US-301 — Provide and store the ChatGPT API token~~

**Status:** Complete

**As a** student,  
**I want** to paste the OpenAI API token we will use,  
**so that** BellaNote can answer questions without me creating an account inside the app.

**Acceptance**

- Header Settings has a single password field for an OpenAI API token.  
- The token is stored in the OS credential store (`com.bellanote.app` / `openai_api_key`): macOS Keychain or Windows Credential Manager, not a plaintext project file.  
- The field can be replaced or cleared. When a token is saved, the field shows a masked placeholder.  
- Chat uses `gpt-4o`. The composer explains that a token is needed until one is saved. Sending without a token opens Settings.  
- We do not display the stored token.

### Scenarios

**First-time save**

- **Given** no token is stored  
- **When** I paste a valid-looking token and save  
- **Then** Settings shows the token as saved (masked placeholder)  
- **And** chat becomes available

**Missing token**

- **Given** no token is stored  
- **When** I open chat and send a question  
- **Then** nothing is sent to OpenAI  
- **And** Settings opens so I can add a token

**Invalid token**

- **Given** a token is stored but OpenAI rejects it  
- **When** I send a question  
- **Then** I see that the key was rejected  
- **And** my question stays in the composer so I can retry after fixing Settings

**Clear token**

- **Given** a token is saved  
- **When** I clear it  
- **Then** chat is disabled again  
- **And** the keychain entry is removed

---

## ~~US-302 — Ask a natural-language question at a chosen scope~~

**Status:** Complete

**As a** student,  
**I want** to ask a question and choose whether BellaNote should use the whole organization, one topic, one meeting group, or one transcript,  
**so that** an answer about Competitive Strategies is not polluted by another class — unless I ask for that.

**Acceptance**

- Chat has a visible scope control: **Org / Topic / Group / This file**.  
- Default follows the tree: org or topic selection uses that level; a meeting group with no loadable file uses the group; a selected file in a group defaults to **This file**.  
- I can widen or narrow before sending (unavailable levels are disabled).  
- Only **Ready** transcripts inside the scope are sent. Audio bytes are never uploaded.  
- If the scope has no Ready transcripts, we do not call the API; a toast says so.  
- Context is packed newest-first up to ~110k characters; older ready files can be omitted (see US-305).  
- The model is instructed to answer only from provided transcripts, cite title + a discrete timestamp, and say when the material does not contain the answer.

### Scenarios

**Happy path — topic scope (the brief’s example)**

- **Given** Competitive Strategies has Lecture Class 1, Lecture Class 2, and Mid Term Study Session with Ready transcripts  
- **And** I set scope to **Topic: Competitive Strategies**  
- **When** I ask *“I think we talked about a new customer onboarding during September. Who was it, and what was the context?”*  
- **Then** the model receives only Ready transcripts under that topic  
- **And** the answer is grounded in those texts  
- **And** if September / onboarding is not there, the answer says it was not found rather than inventing a customer

**Org scope**

- **Given** Duke also has Corporate Finance transcripts  
- **And** I set scope to **Organization: Duke**  
- **When** I ask *“What exams or deliverables were mentioned this month?”*  
- **Then** both courses may be used  
- **And** the answer distinguishes which topic / group a point came from

**Meeting group scope**

- **Given** I set scope to **Meeting group: Lecture Class 1**  
- **When** I ask *“What was the definition of switching costs?”*  
- **Then** only artifacts in Lecture Class 1 are used  
- **And** Lecture Class 2 is not used even if it discussed the same idea

**Single transcript scope**

- **Given** I have `zoom-export.vtt` open  
- **And** I set scope to **This file**  
- **When** I ask *“What questions did students ask at the end?”*  
- **Then** only that file is used

**Scope has nothing Ready**

- **Given** Lecture Class 1 only has an audio file still Transcribing  
- **And** I set scope to that meeting group  
- **When** I send a question  
- **Then** no API call is made  
- **And** I am told there is no ready transcript in this scope yet

**Scope includes a mix of Ready and Failed**

- **Given** Lecture Class 1 has one Ready import and one Failed audio  
- **When** I chat at meeting-group scope  
- **Then** only the Ready import is used  
- **And** the preview notes ready vs not-ready counts

**Empty question**

- **Given** the composer is blank or whitespace  
- **When** I press send  
- **Then** nothing is sent

**Offline**

- **Given** the Mac has no internet  
- **When** I send a question  
- **Then** I am told chat needs a network connection  
- **And** my hierarchy and transcripts remain available

**Rate limit / API error**

- **Given** OpenAI returns a rate-limit or server error  
- **When** my request fails  
- **Then** I see a retryable error  
- **And** the conversation is not replaced with a fake answer

---

## US-303 — Read a well-organized answer

**Status:** Partial

**As a** student,  
**I want** the reply laid out so I can scan it, see sources, and jump back to the file,  
**so that** I trust the answer and can verify it in the original transcript.

**Acceptance**

- Assistant messages render GitHub-flavored markdown (headings, lists, quotes, tables) — not a raw dump.  
- User and assistant bubbles are visually distinct, with a name and timestamp.  
- While waiting, a thinking bubble shows.  
- The model is asked to cite `[00:17]` and the artifact title in the prose. Those cites are plain text, not clickable chips.  
- Long answers scroll inside the chat column.

**Still open**

- No structured source list (title / group / topic / timestamp).  
- Clicking a citation does not open the artifact or seek the waveform.

### Scenarios

**Answer with citations**

- **Given** the model cites `zoom-export.vtt` at `[00:17]`  
- **When** the answer renders  
- **Then** I see that citation in the markdown  
- **And** I open the file myself from the Files table to verify

**Answer with no match**

- **Given** the transcripts do not mention the thing I asked  
- **When** the answer returns  
- **Then** it states that clearly  
- **And** it may suggest widening or narrowing the scope

**Ugly model markdown**

- **Given** the model returns messy markdown  
- **When** it renders  
- **Then** we still get readable text (markdown rendered, not shown as backticks soup)

---

## US-304 — Continue the conversation in the same scope

**Status:** Partial

**As a** student,  
**I want** to ask a follow-up without restating the whole question,  
**so that** I can dig in (“who said that?” / “what was the example?”) naturally.

**Acceptance**

- Follow-ups stay in the same locally persisted thread for the current scope (`UNIQUE(scope_type, scope_id)`).  
- Changing the scope chips loads that scope’s own thread (no in-thread “scope changed” divider).  
- **New thread** clears the current scope’s thread. The old messages are not kept as a history list.

**Still open**

- No list of recent threads per scope.  
- New thread replaces the saved thread; it does not archive it.

### Scenarios

**Follow-up**

- **Given** I asked about switching costs at Lecture Class 1 scope  
- **When** I ask *“Can you quote the example they used?”*  
- **Then** the model still only uses Lecture Class 1  
- **And** it can refer to the previous answer

**Change scope mid-conversation**

- **Given** I was chatting at Lecture Class 1  
- **When** I switch scope to Competitive Strategies  
- **Then** I see that topic’s thread (or an empty one)  
- **And** the next send uses the wider scope

**New thread**

- **Given** I have a long thread about the midterm  
- **When** I click New thread  
- **Then** the old thread is not sent as context  
- **And** it is no longer listed

**Thread persistence**

- **Given** I chatted yesterday about Lecture Class 1  
- **When** I reopen that group today and set scope to the group  
- **Then** the last thread for that scope is still there

---

## US-305 — See what will be included before I send (scope preview)

**Status:** Partial

**As a** student,  
**I want** to see how many transcripts, and which ones, the current scope will use,  
**so that** I do not accidentally ask “all of Duke” when I meant one lecture.

**Acceptance**

- Under the scope chips, a count line shows ready / omitted / not-ready, e.g. `3 ready · 1 not ready` or `12 ready · using 8 most recent · 4 omitted`.  
- Context is newest meeting groups first, packed to about 110k characters of transcript text.  
- Omitted ready files are mentioned to the model; the UI shows the omitted count.

**Still open**

- The preview is counts only — no expandable list of titles or Ready/excluded labels, even though the API already returns that file list.

### Scenarios

**Preview before send**

- **Given** Competitive Strategies has 3 Ready artifacts and 1 Importing  
- **When** I set scope to that topic  
- **Then** I see `3 ready · 1 not ready`

**Org-wide surprise**

- **Given** Duke has more ready text than the context budget  
- **When** I set scope to Duke  
- **Then** the preview shows how many ready files will be used and how many older ones are omitted  
- **And** I can narrow the scope if I want a specific lecture included

---

# Cross-cutting stories

## ~~US-401 — Work stays on this Mac~~

**Status:** Complete

**As a** student,  
**I want** my audio and transcripts to stay on my computer,  
**so that** organizing class recordings does not mean uploading Duke material to BellaNote’s servers.

**Acceptance**

- Hierarchy, files, and transcripts live under `~/Library/Application Support/com.bellanote.app/` (`bella.db` + `library/{id}/`).  
- The only network calls in this slice are OpenAI chat (and only when I send a message).  
- Audio is not uploaded to OpenAI; we send transcript text (and only for the chosen scope).

### Scenarios

**Chat sends text, not audio**

- **Given** Lecture Class 1 has `lecture-1.m4a` and its Ready transcript  
- **When** I ask a question at group scope  
- **Then** the request to ChatGPT contains transcript text, not the audio bytes

---

## ~~US-402 — The workspace looks like BellaNote~~

**Status:** Complete

**As a** student,  
**I want** the app to feel simple and beautiful,  
**so that** I can find Duke → Competitive Strategies → Lecture Class 1 without learning a complex tool.

**Acceptance**

- Three panes: Library | Transcript workspace | Chat.  
- Library, transcript, and chat each collapse to a vertical rail. Collapse state persists. When the transcript rail is collapsed, Chat grows; when Chat is collapsed, the transcript takes the leftover space. Titlebar layout buttons focus one pane; widths animate.  
- Chat is a first-class pane, not a modal.  
- Empty states teach the next click.  
- Dark charcoal / mint glass is the shipped theme (no light theme in this slice).  
- Meeting-group workspace uses stacked Files and Transcript cards, not a single dump. The Transcript header uses Audio / Transcript chips instead of a static “Audio + transcript” label. Audio controls sit under the waveform; transcript search sits under the lines.

### Scenarios

**First-run teaching**

- **Given** a new install  
- **When** I arrive  
- **Then** I understand: create an organization → add a topic → add a meeting group → add files → ask

**Chat + transcript together**

- **Given** I am reading `zoom-export.vtt` and chatting at This file  
- **When** an answer arrives  
- **Then** I can see the transcript and the answer without losing my place

**Collapse chat**

- **Given** Chat is open  
- **When** I collapse it  
- **Then** it becomes a Chat rail  
- **And** the transcript workspace uses the extra width  
- **And** the panes animate to those widths

**Focus chat from the titlebar**

- **Given** Library, Transcript, and Chat are open  
- **When** I click **Show chat only**  
- **Then** Library and Transcript become rails  
- **And** Chat fills the leftover space

---

# Decisions (resolved in this slice)

| # | Decision | Shipped | Notes |
|---|---|---|---|
| D1 | Must every topic live under an organization? | **Yes** | Unfiled topics are out. |
| D2 | Must every meeting group live under a topic? | **Yes** | Same tree. |
| D3 | Duplicate names in the same parent | **Block org and topic (case-insensitive); allow meeting groups** | Groups are distinguished by date. |
| D4 | Delete original files on disk? | **Optional, off by default** | Confirm dialog checkbox. BellaNote library copies of imports are still removed; recordings stay on disk unless the box is checked. |
| D5 | Video (`mp4`) in this slice | **Import MP4 via Import Meeting → Video files** | Keeps `video.mp4`, extracts audio once (PyAV in sidecar), Watch opens a video window. Non-MP4 video still rejected. |
| D6 | Undo after delete | **No undo; confirm instead** | Confirm always, including empty containers. |
| D7 | Org-wide chat when there are dozens of transcripts | **Newest groups first, ~110k character budget, show omitted count** | Expandable file list is still open (US-305). |
| D8 | Playback of source audio | **Shipped** | Waveform with click-to-seek; play, Follow, speed (1x–2x), and playhead/total clock on a row under the waveform. |
| D9 | Persist chat threads per scope | **Yes — one thread per scope** | New thread replaces; no archive list (US-304). |
| D10 | Independent org/topic tags (from product scope) | **Not in this slice** | Real tree: Org → Topic → Group → Artifact. |

---

# Suggested next work

Shipped: US-101–107, US-201–204, US-206, US-207, US-209–211, US-301, US-302, US-401, US-402.

Still open from this slice:

1. **US-208** — Timed comments on audio (this branch)  
2. **US-205** — Move an artifact to another meeting group  
3. **US-303** — Clickable citations that open the artifact / seek  
4. **US-304** — Thread archive / list of recent threads  
5. **US-305** — Expandable scope-preview file list  

Then the product-scope work that was always out of this slice: summaries, tasks, YouTube, sharing.

---

# Traceability to the Duke example

| You said | Stories |
|---|---|
| Organization = Duke | US-102, US-105 |
| Topic = Competitive Strategies | US-103, US-105 |
| Meeting groups = Lecture Class 1 / 2, Mid Term Study Session | US-104 |
| Upload audio provided after the fact | US-201, US-202 |
| Recordings I made while I was there (as files) | US-201 |
| Record live from this Mac or Windows PC | US-209, US-210 |
| Transcripts downloaded from Zoom | US-203 |
| Find a phrase in a long transcript | US-207 |
| Pin a thought to a moment in the lecture | US-208 |
| Ask NL questions with org / topic / group / one-file scope | US-302, US-305 |
| Well-organized answer UI | US-303, US-402 |
| ChatGPT API token we provide | US-301 |
