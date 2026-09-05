# BellaNote — First-slice user stories

**Document status:** Ready for comment  
**Platform:** macOS desktop app  
**Slice goal:** A student (or anyone with a similar hierarchy) can create an organization and topic categories, file audio and existing transcripts into meeting groups, and ask natural-language questions against a chosen scope.

This slice is narrower than the full [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md). Live microphone / system-audio capture, Windows, summaries, task extraction, and a sharing backend are **out**. The Duke example below is the north-star walkthrough.

---

## How to read this

Each story uses:

- **As a / I want / so that** — the user-facing intent  
- **Acceptance** — what “done” looks like  
- **Scenarios** — Given / When / Then, including edges we should decide before building

Comment in the **Open decisions** boxes and on any scenario marked **DECIDE**. Those are the places the product can go either way.

---

## North-star walkthrough (Duke)

1. Launch BellaNote on a Mac.  
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
| **Chat scope** | How wide the model is allowed to look | Org / topic / meeting group / one transcript |

**Refinement vs. product scope:** [`PRODUCT_SCOPE.md`](./PRODUCT_SCOPE.md) treated each recording as a standalone meeting, with org and topic as optional independent tags. This slice introduces **meeting group** as a first-class folder of artifacts, and treats the intended path as **Org → Topic → Group → Artifact**. Optional / unfiled items are still covered as edge cases.

---

## Slice in / out

| In | Out |
|---|---|
| Launch on Mac | Windows |
| Create / rename / delete org, topic, meeting group | Live mic or system+meeting capture |
| Upload audio → transcribe on device | YouTube URL ingest |
| Import existing transcript files | Audience-aware summaries, task extraction |
| Place, move, and view artifacts in the hierarchy | Sharing / multi-user backend |
| ChatGPT chat with an explicit scope | Calendar, speaker diarization |
| User-provided OpenAI API token | Mobile |

---

## Story map

```
Epic 1  Launch & hierarchy
        US-101  Launch the Mac app
        US-102  Create an organization
        US-103  Create a topic category
        US-104  Create a meeting group
        US-105  Browse the hierarchy
        US-106  Rename containers
        US-107  Delete containers

Epic 2  Files & transcripts
        US-201  Add an audio file to a meeting group
        US-202  Watch transcription finish
        US-203  Import an existing transcript
        US-204  Open and read an artifact
        US-205  Re-file / move an artifact
        US-206  Remove an artifact

Epic 3  Scoped chat
        US-301  Save the ChatGPT API token
        US-302  Ask a question against a chosen scope
        US-303  Read a well-organized answer
        US-304  Continue a conversation
        US-305  See what the model used (and did not use)
```

---

# Epic 1 — Launch the Mac app and build the hierarchy

## US-101 — Launch BellaNote on a Mac

**As a** student on a Mac,  
**I want** to open BellaNote like any other desktop app,  
**so that** I can start organizing class material without a browser or account.

**Acceptance**

- A standard macOS app launches to a local workspace.  
- No sign-in is required.  
- First launch shows an empty state that explains the next step (create an organization).  
- Closing and reopening restores organizations, topics, groups, and artifacts.

### Scenarios

**Happy path — first launch**

- **Given** BellaNote is installed and has never been opened  
- **When** I launch it from Applications or Spotlight  
- **Then** I see a calm empty workspace and a primary action to create an organization  
- **And** I am not asked for an account or cloud login

**Relaunch restores work**

- **Given** I previously created Duke / Competitive Strategies / Lecture Class 1  
- **When** I quit and launch again  
- **Then** that hierarchy is still there, with the last place I was looking selected if possible

**Already running**

- **Given** BellaNote is already open  
- **When** I launch it again  
- **Then** the existing window comes forward (no second empty workspace)

**Permissions (this slice)**

- **Given** I have not yet added audio  
- **When** the app launches  
- **Then** it does not demand Microphone or Screen Recording permission  
- **And** it only asks for file access when I pick a file

---

## US-102 — Create an organization

**As a** student,  
**I want** to create an organization named after my school,  
**so that** all of my Duke material lives in one place.

**Acceptance**

- I can create an organization with a name.  
- The new org appears immediately in the library.  
- Names are trimmed; empty names are rejected.  
- I can have more than one organization (e.g. Duke and a summer internship).

### Scenarios

**Happy path**

- **Given** I am in the empty or existing library  
- **When** I create an organization named `Duke`  
- **Then** Duke appears as a top-level item  
- **And** I can open it and see that it has no topics yet

**Whitespace-only name**

- **Given** the create-organization field is open  
- **When** I enter `   ` and confirm  
- **Then** the org is not created  
- **And** I see a short message that a name is required

**Duplicate name — DECIDE**

- **Given** Duke already exists  
- **When** I try to create another organization named `Duke` (or `duke`)  
- **Then** *(proposed)* the app blocks it and asks me to pick a different name  
- **And** we treat names as case-insensitive for uniqueness

**Very long name**

- **Given** I paste a 300-character name  
- **When** I save  
- **Then** the app either truncates at a documented limit (proposed: 80 characters) or rejects with a clear reason  
- **And** the library row does not break the layout

---

## US-103 — Create a topic category

**As a** Duke student,  
**I want** to add a topic category for a class,  
**so that** Competitive Strategies is distinct from my other courses.

**Acceptance**

- From an organization, I can create a topic category with a name.  
- The topic appears nested under that organization.  
- A topic is not required to contain meeting groups yet.

### Scenarios

**Happy path**

- **Given** organization Duke exists  
- **When** I create a topic category `Competitive Strategies` under Duke  
- **Then** it appears under Duke  
- **And** opening it shows an empty list of meeting groups and an action to create one

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

**Duplicate topic in the same org — DECIDE**

- **Given** Duke already has Competitive Strategies  
- **When** I create another topic named `Competitive Strategies` under Duke  
- **Then** *(proposed)* the app blocks the duplicate inside that org

**Topic without an organization — DECIDE**

- **Given** I have not created any organization  
- **When** I look for a way to create a topic  
- **Then** *(proposed for this slice)* I must create an organization first  
- **And** the empty state says so  
- **Comment asked:** later we may allow unfiled topics; this slice can stay hierarchical to match Duke.

---

## US-104 — Create a meeting group

**As a** student in Competitive Strategies,  
**I want** to create named meeting groups for each lecture or study session,  
**so that** several recordings and transcripts from the same class meeting live together.

**Acceptance**

- From a topic, I can create a meeting group with a name.  
- Optional date (defaults to today) can be set at creation or edited later.  
- The group can be empty (no files yet).

### Scenarios

**Happy path**

- **Given** I am in Duke → Competitive Strategies  
- **When** I create meeting groups `Lecture Class 1`, `Lecture Class 2`, and `Mid Term Study Session`  
- **Then** all three appear under the topic  
- **And** each can be opened and shows an empty file list plus “Add audio” and “Add transcript”

**Default date**

- **Given** today is 5 September 2026  
- **When** I create Lecture Class 1 without picking a date  
- **Then** the group is dated today  
- **And** I can later change that date

**Duplicate group name in the same topic — DECIDE**

- **Given** Lecture Class 1 already exists in Competitive Strategies  
- **When** I create another `Lecture Class 1` in the same topic  
- **Then** *(proposed)* allowed if I really want two (e.g. two sections), but the UI distinguishes them by date  
- **Or** we block duplicates — needs a call

**Group without a topic — DECIDE**

- **Given** I am looking at Duke with no topic selected  
- **When** I try to create a meeting group  
- **Then** *(proposed for this slice)* I must pick or create a topic first

---

## US-105 — Browse the hierarchy

**As a** student,  
**I want** to move from Duke down to a single lecture group in a few clicks,  
**so that** I always know where I am and what files belong together.

**Acceptance**

- Library shows Org → Topic → Meeting group.  
- Selecting a level shows only that level’s children.  
- Breadcrumb or equivalent always shows the path (e.g. Duke / Competitive Strategies / Lecture Class 1).  
- Empty levels have a useful empty state, not a blank panel.

### Scenarios

**Drill in**

- **Given** the Duke example data exists  
- **When** I click Duke, then Competitive Strategies, then Lecture Class 1  
- **Then** I see only the artifacts in Lecture Class 1  
- **And** the path Duke / Competitive Strategies / Lecture Class 1 is visible

**Org with many topics**

- **Given** Duke has 12 topic categories  
- **When** I open Duke  
- **Then** the list is scrollable and readable  
- **And** I can find Competitive Strategies without renaming anything

**Empty topic**

- **Given** I just created Competitive Strategies  
- **When** I open it  
- **Then** I see “No meeting groups yet” and a create action  
- **And** I do not see other topics’ groups

---

## US-106 — Rename a container

**As a** student,  
**I want** to rename an organization, topic, or meeting group,  
**so that** a typo or a better name does not force me to recreate the tree.

**Acceptance**

- Rename in place or via a simple edit.  
- Children stay attached after rename.  
- Same uniqueness rules as create.

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

## US-107 — Delete a container

**As a** student,  
**I want** to delete an organization, topic, or meeting group I no longer need,  
**so that** my library stays tidy.

**Acceptance**

- Delete asks for confirmation when the container is not empty.  
- I understand what will be removed (children + artifacts).  
- Cancel leaves everything unchanged.

### Scenarios

**Delete empty group**

- **Given** Lecture Class 1 has no artifacts  
- **When** I delete it and confirm  
- **Then** it disappears from Competitive Strategies

**Delete group with files**

- **Given** Lecture Class 1 contains two audio transcripts and one imported VTT  
- **When** I choose delete  
- **Then** I am told those 3 artifacts will be removed from BellaNote  
- **And** I must confirm  
- **And** *(proposed)* the original files on disk are **not** deleted — only BellaNote’s copy / reference

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

**Undo — DECIDE**

- **Given** I just deleted a meeting group  
- **When** I look for undo  
- **Then** *(proposed for this slice)* no undo; confirmation is the safety net  
- **Comment asked:** the POC had undo-on-delete for meetings. Worth matching?

---

# Epic 2 — Add files and organize transcripts

## US-201 — Add an audio file to a meeting group

**As a** student,  
**I want** to choose an audio file from my Mac and attach it to a meeting group,  
**so that** a posted lecture recording becomes part of Lecture Class 1.

**Acceptance**

- From a meeting group I can pick one or more audio files (at least wav, mp3, m4a, aac, ogg, flac).  
- Each file becomes an artifact in that group with the original filename as the default title (editable).  
- Transcription starts automatically after a successful add.  
- Unsupported types are rejected with a readable reason.

### Scenarios

**Happy path — professor’s recording**

- **Given** I am in Duke / Competitive Strategies / Lecture Class 1  
- **When** I choose `lecture-1.m4a` from Downloads  
- **Then** the artifact appears in the group immediately  
- **And** its status is “Transcribing”  
- **And** I can keep using the app

**Phone recording from class**

- **Given** I am in the same meeting group  
- **When** I add `voice-memo.wav` from my phone’s synced folder  
- **Then** it appears as a second artifact beside `lecture-1.m4a`  
- **And** both can transcribe independently

**Unsupported file**

- **Given** I pick `slides.pptx` or `photo.jpg`  
- **When** I confirm the file picker  
- **Then** the file is not added  
- **And** I see that BellaNote needs audio or a transcript file

**Video file — DECIDE**

- **Given** I pick `lecture.mp4`  
- **When** I add it  
- **Then** *(proposed)* BellaNote extracts audio only and transcribes that  
- **Or** rejects video in this slice to keep scope tight

**Duplicate of the same file**

- **Given** `lecture-1.m4a` is already in Lecture Class 1  
- **When** I add the same path again  
- **Then** *(proposed)* we allow it as a second artifact only after a “this looks like a duplicate” confirm  
- **Or** we silently ignore — needs a call

**File disappears after pick**

- **Given** I selected a file on a USB drive  
- **When** the drive is ejected before copy/transcribe finishes  
- **Then** the artifact is marked failed with “Couldn’t read the file”  
- **And** I can remove it or try again

**Very large file**

- **Given** I add a 3-hour, 1 GB lecture  
- **When** transcription starts  
- **Then** the UI stays usable  
- **And** progress does not look frozen (time or percent, even if approximate)

---

## US-202 — See transcription complete (or fail) and retry

**As a** student,  
**I want** to know when an audio file has become a transcript I can read and chat with,  
**so that** I am not guessing whether the file is ready.

**Acceptance**

- Status values: Queued, Transcribing, Ready, Failed.  
- Ready artifacts open to the transcript text.  
- Failed artifacts show a short reason and Retry.  
- Chat will not silently include artifacts that are not Ready.

### Scenarios

**Success**

- **Given** `lecture-1.m4a` was added  
- **When** local transcription finishes  
- **Then** status becomes Ready  
- **And** I can open the transcript  
- **And** the artifact is eligible for chat

**Failure — unreadable audio**

- **Given** the file is corrupt or silent/unusable  
- **When** transcription fails  
- **Then** status is Failed  
- **And** I see a human sentence, not a stack trace  
- **And** Retry is available

**Quit mid-transcribe**

- **Given** transcription is in progress  
- **When** I quit the app  
- **Then** on next launch the artifact is Failed or Queued (not stuck on Transcribing forever)  
- **And** I can retry

**Two files at once**

- **Given** I added two audio files  
- **When** both are transcribing  
- **Then** each shows its own status  
- **And** one failure does not cancel the other

---

## US-203 — Import an existing transcript file

**As a** student,  
**I want** to add a transcript I already have (Zoom, Teams, or a plain text file) to a meeting group,  
**so that** I can organize it next to recordings without re-transcribing.

**Acceptance**

- From a meeting group I can pick transcript files: `.vtt`, `.srt`, `.txt`, and at least one Zoom/Teams-style export if we can detect it.  
- Audio is not required.  
- Imported text is stored as a Ready transcript.  
- Timestamps are kept when the file has them.

### Scenarios

**Happy path — Zoom download**

- **Given** I am in Lecture Class 1  
- **When** I import `zoom-export.vtt`  
- **Then** the artifact appears as Ready (no transcribe wait)  
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

**Wrong extension, right content — DECIDE**

- **Given** a Zoom transcript saved as `.txt`  
- **When** I import it via “Add transcript”  
- **Then** it is treated as plain text (acceptable)

**Audio mistaken for transcript**

- **Given** I choose `lecture-1.m4a` in the transcript picker  
- **When** I confirm  
- **Then** BellaNote either rejects it or offers “Add as audio instead”

**Huge text file**

- **Given** a 20 MB dump  
- **When** I import  
- **Then** *(proposed)* we accept up to a documented cap (e.g. 2 MB of text) and reject above it with a reason

---

## US-204 — Open an artifact and see its transcript

**As a** student,  
**I want** to click an artifact and read the transcript in a readable layout,  
**so that** I can skim a lecture before I ask questions.

**Acceptance**

- Ready artifacts open in the main workspace.  
- Title, source type (Audio / Imported), date, and path in the hierarchy are visible.  
- Long transcripts scroll smoothly.  
- If audio exists, I can play it (nice-to-have in this slice if cheap; required later).

### Scenarios

**Read imported VTT**

- **Given** `zoom-export.vtt` is Ready  
- **When** I open it  
- **Then** I see the text in reading order  
- **And** timestamps display if present

**Open while still transcribing**

- **Given** `lecture-1.m4a` is Transcribing  
- **When** I open it  
- **Then** I see progress, not a fake empty transcript

**Rename artifact**

- **Given** the title is `lecture-1.m4a`  
- **When** I rename it to `Lecture 1 — professor recording`  
- **Then** the library and chat source list use the new title

---

## US-205 — Move an artifact to another meeting group

**As a** student,  
**I want** to move a file I dropped in the wrong group,  
**so that** organization stays accurate without re-importing.

**Acceptance**

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

## US-206 — Remove an artifact

**As a** student,  
**I want** to remove a file I added by mistake,  
**so that** it no longer appears in the group or in chat.

**Acceptance**

- Remove asks for confirmation.  
- *(Proposed)* original disk file is left alone.  
- Removed artifacts are out of every chat scope immediately.

### Scenarios

**Remove imported transcript**

- **Given** `zoom-export.vtt` is in Lecture Class 1  
- **When** I remove it and confirm  
- **Then** the group no longer lists it  
- **And** a topic-scoped chat no longer uses that text

**Remove during transcribe**

- **Given** an audio file is Transcribing  
- **When** I remove it and confirm  
- **Then** work on that file stops  
- **And** it is gone from the group

---

# Epic 3 — Chat with a chosen scope

## US-301 — Provide and store the ChatGPT API token

**As a** student,  
**I want** to paste the OpenAI API token we will use,  
**so that** BellaNote can answer questions without me creating an account inside the app.

**Acceptance**

- Settings has a single field for an OpenAI API token.  
- The token is stored in the macOS keychain, not in a plaintext project file.  
- The field can be replaced or cleared.  
- Chat is disabled with a clear explanation until a token is saved.  
- We do not log the token in the UI or in local debug output.

### Scenarios

**First-time save**

- **Given** no token is stored  
- **When** I paste a valid-looking token and save  
- **Then** Settings shows the token as saved (masked)  
- **And** chat becomes available

**Missing token**

- **Given** no token is stored  
- **When** I open chat and send a question  
- **Then** nothing is sent to OpenAI  
- **And** I am pointed to Settings to add a token

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

## US-302 — Ask a natural-language question at a chosen scope

**As a** student,  
**I want** to ask a question and choose whether BellaNote should use the whole organization, one topic, one meeting group, or one transcript,  
**so that** an answer about Competitive Strategies is not polluted by another class — unless I ask for that.

**Acceptance**

- Chat has a visible **scope control** with four levels: Organization, Topic category, Meeting group, This transcript.  
- Scope defaults to wherever I am (if I am inside Lecture Class 1, default is that group).  
- I can widen or narrow before sending.  
- Only **Ready** transcripts inside the scope are sent.  
- If the scope has no Ready transcripts, we do not call the API; we say so.  
- The model is instructed to answer only from provided transcripts, cite sources, and say when the material does not contain the answer.

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
- **And** I set scope to **This transcript**  
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
- **And** the UI notes that 1 of 2 artifacts could be included

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

**As a** student,  
**I want** the reply laid out so I can scan it, see sources, and jump back to the file,  
**so that** I trust the answer and can verify it in the original transcript.

**Acceptance**

- Assistant messages use readable typography (headings, bullets, short paragraphs) — not a raw dump.  
- Sources appear as a structured list: artifact title, meeting group, topic, and timestamp when we have one.  
- Clicking a source opens that artifact (and scrolls to a timestamp if we have one).  
- User question and assistant answer are visually distinct.  
- Long answers scroll inside the chat column; they do not blow up the library.

### Scenarios

**Answer with citations**

- **Given** the model cites Lecture Class 1 / `zoom-export.vtt` at `00:17`  
- **When** the answer renders  
- **Then** I see a source chip or footnote I can click  
- **And** clicking opens that transcript near 00:17 if timestamps exist

**Answer with no match**

- **Given** the transcripts do not mention the thing I asked  
- **When** the answer returns  
- **Then** it states that clearly  
- **And** it may suggest widening the scope (e.g. from this group to the whole topic)

**Ugly model markdown**

- **Given** the model returns messy markdown  
- **When** it renders  
- **Then** we still get readable text (markdown rendered, not shown as backticks soup)

---

## US-304 — Continue the conversation in the same scope

**As a** student,  
**I want** to ask a follow-up without restating the whole question,  
**so that** I can dig in (“who said that?” / “what was the example?”) naturally.

**Acceptance**

- Follow-ups stay in the same thread and keep the current scope unless I change it.  
- Changing scope mid-thread is visible (a divider or notice: “Scope changed to Topic: Competitive Strategies”).  
- I can start a new thread so an old tangent does not poison the next exam question.

### Scenarios

**Follow-up**

- **Given** I asked about switching costs at Lecture Class 1 scope  
- **When** I ask *“Can you quote the example they used?”*  
- **Then** the model still only uses Lecture Class 1  
- **And** it can refer to the previous answer

**Change scope mid-thread**

- **Given** I was chatting at Lecture Class 1  
- **When** I switch scope to Competitive Strategies and send the next message  
- **Then** that message uses the wider scope  
- **And** the thread shows that the scope changed

**New thread**

- **Given** I have a long thread about the midterm  
- **When** I start a new chat  
- **Then** the old thread is not sent as context  
- **And** I can still find the old thread later *(proposed: list of recent threads on this scope)*

**Thread persistence**

- **Given** I chatted yesterday about Lecture Class 1  
- **When** I reopen that group today  
- **Then** *(proposed)* the last thread is still there

---

## US-305 — See what will be included before I send (scope preview)

**As a** student,  
**I want** to see how many transcripts, and which ones, the current scope will use,  
**so that** I do not accidentally ask “all of Duke” when I meant one lecture.

**Acceptance**

- Near the scope control, show a count: e.g. “3 transcripts in Competitive Strategies”.  
- I can expand the list and see titles + Ready/excluded.  
- Excluded items (still transcribing, failed, empty) are labeled.

### Scenarios

**Preview before send**

- **Given** Competitive Strategies has 3 Ready artifacts and 1 Transcribing  
- **When** I set scope to that topic  
- **Then** I see “3 ready · 1 not ready”  
- **And** I can open the list and confirm Lecture Class 1 and 2 are included

**Org-wide surprise**

- **Given** Duke has 40 Ready transcripts  
- **When** I set scope to Duke  
- **Then** the preview shows 40  
- **And** *(proposed)* if we must cap context, we tell the user we will use the most recent N and list which ones  
- **DECIDE:** hard cap strategy (most recent by group date vs. ask the user to narrow)

---

# Cross-cutting stories

## US-401 — Work stays on this Mac

**As a** student,  
**I want** my audio and transcripts to stay on my computer,  
**so that** organizing class recordings does not mean uploading Duke material to BellaNote’s servers.

**Acceptance**

- Hierarchy, files, and transcripts are local.  
- The only network calls in this slice are OpenAI chat (and only when I send a message).  
- Audio is not uploaded to OpenAI; we send transcript text (and only for the chosen scope).

### Scenarios

**Chat sends text, not audio**

- **Given** Lecture Class 1 has `lecture-1.m4a` and its Ready transcript  
- **When** I ask a question at group scope  
- **Then** the request to ChatGPT contains transcript text, not the audio bytes

---

## US-402 — The workspace looks like BellaNote

**As a** student,  
**I want** the app to feel simple and beautiful,  
**so that** I can find Duke → Competitive Strategies → Lecture Class 1 without learning a complex tool.

**Acceptance**

- Library on one side, reading / chat on the other.  
- Chat output is a first-class pane, not a cramped modal.  
- Empty states teach the next click.  
- Light and dark (or the POC dark default) do not make text unreadable.

### Scenarios

**First-run teaching**

- **Given** a new install  
- **When** I arrive  
- **Then** I understand: create an organization → add a topic → add a meeting group → add files → ask

**Chat + transcript together**

- **Given** I am reading `zoom-export.vtt` and chatting at This transcript  
- **When** an answer cites a line  
- **Then** I can see the transcript and the answer without losing my place

---

# Open decisions (please comment)

| # | Decision | Proposed default | Why it matters |
|---|---|---|---|
| D1 | Must every topic live under an organization? | **Yes, for this slice** | Matches Duke. Unfiled topics can wait. |
| D2 | Must every meeting group live under a topic? | **Yes, for this slice** | Same reason. |
| D3 | Duplicate names in the same parent | **Block for org and topic; allow for meeting groups if dates differ** | Classes repeat; orgs should not. |
| D4 | Delete original files on disk? | **Never** | We organize copies/references, not the user’s Downloads folder. |
| D5 | Video (`mp4`) in this slice | **Reject, ask for audio** | Keeps ingest small. Extract-audio can be next. |
| D6 | Undo after delete | **No undo; confirm instead** | Faster to ship. POC had undo — we can match it if you want. |
| D7 | Org-wide chat when there are dozens of transcripts | **Warn + use most recent N (e.g. 20), show which** | Context windows are real. |
| D8 | Playback of source audio in this slice | **Nice-to-have if the POC playback ports cheaply; not blocking** | Chat and read-first. |
| D9 | Persist chat threads per scope | **Yes, locally** | Students will come back the night before the midterm. |
| D10 | Independent org/topic tags (from product scope) | **Not in this slice** | This slice is a real tree: Org → Topic → Group → Artifact. |

---

# Suggested build order

1. **US-101 → US-105** — empty Mac app, tree, browse, persist  
2. **US-106, US-107** — rename / delete with confirmations  
3. **US-201, US-202** — audio add + local transcribe + status  
4. **US-203, US-204** — import transcript + reader  
5. **US-205, US-206** — move / remove  
6. **US-301** — keychain token  
7. **US-305 then US-302 → US-304** — preview, ask, layout, follow-up  

That order means you can comment on hierarchy and file edges before we spend time on chat formatting.

---

# Traceability to the Duke example

| You said | Stories |
|---|---|
| Organization = Duke | US-102, US-105 |
| Topic = Competitive Strategies | US-103, US-105 |
| Meeting groups = Lecture Class 1 / 2, Mid Term Study Session | US-104 |
| Upload audio provided after the fact | US-201, US-202 |
| Recordings I made while I was there (as files) | US-201 |
| Transcripts downloaded from Zoom | US-203 |
| Ask NL questions with org / topic / group / one-file scope | US-302, US-305 |
| Well-organized answer UI | US-303, US-402 |
| ChatGPT API token we provide | US-301 |
