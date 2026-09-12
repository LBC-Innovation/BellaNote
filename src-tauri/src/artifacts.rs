use crate::db::Artifact;
use crate::error::{AppError, AppResult};
use crate::import_transcript::parse_transcript_file;
use crate::paths;
use crate::state::AppState;
use crate::transcribe::Transcriber;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const AUDIO_EXTS: &[&str] = &["wav", "mp3", "m4a", "aac", "ogg", "flac", "webm"];
const TRANSCRIPT_EXTS: &[&str] = &["vtt", "srt", "txt"];
const VIDEO_EXTS: &[&str] = &["mp4", "mov", "mkv", "avi"];
/// Import Meeting → Video files accepts MP4 only in this slice.
const IMPORT_VIDEO_EXTS: &[&str] = &["mp4"];

fn ext_of(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .chars()
        .take(80)
        .collect()
}

fn copy_into_library(app: &AppHandle, artifact_id: &str, source: &Path) -> AppResult<PathBuf> {
    let dir = paths::artifact_dir(app, artifact_id)?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Message(e.to_string()))?;
    let ext = ext_of(source);
    let dest = dir.join(format!("source.{ext}"));
    std::fs::copy(source, &dest).map_err(|e| AppError::Message(format!("Could not copy file: {e}")))?;
    Ok(dest)
}

fn copy_video_into_library(app: &AppHandle, artifact_id: &str, source: &Path) -> AppResult<PathBuf> {
    let dir = paths::artifact_dir(app, artifact_id)?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Message(e.to_string()))?;
    let ext = ext_of(source);
    let dest = dir.join(format!("video.{ext}"));
    std::fs::copy(source, &dest).map_err(|e| AppError::Message(format!("Could not copy file: {e}")))?;
    Ok(dest)
}

pub fn import_audio(app: &AppHandle, state: &Arc<AppState>, meeting_group_id: &str, path: &str) -> AppResult<Artifact> {
    let source = PathBuf::from(path);
    let ext = ext_of(&source);
    if VIDEO_EXTS.contains(&ext.as_str()) {
        return Err(AppError::Message(
            "That looks like video. Use Import Meeting → Video files for MP4.".into(),
        ));
    }
    if !AUDIO_EXTS.contains(&ext.as_str()) {
        return Err(AppError::Message(
            "BellaNote needs an audio file (wav, mp3, m4a, aac, ogg, or flac).".into(),
        ));
    }
    if !source.is_file() {
        return Err(AppError::Message("Couldn’t read the file.".into()));
    }

    let id = Uuid::new_v4().to_string();
    let dest = copy_into_library(app, &id, &source)?;
    let filename = source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("audio")
        .to_string();
    let artifact = Artifact {
        id: id.clone(),
        meeting_group_id: meeting_group_id.to_string(),
        title: title_from_path(&source),
        source_type: "audio_upload".into(),
        status: "queued".into(),
        has_audio: true,
        original_filename: filename,
        original_path: source.to_string_lossy().into_owned(),
        error_message: String::new(),
        transcript: String::new(),
        segments_json: "[]".into(),
        duration_ms: 0,
        created_at: Utc::now().to_rfc3339(),
        whisper_model: String::new(),
    };
    state.db.insert_artifact(&artifact)?;
    spawn_transcribe(app.clone(), Arc::clone(state), id, dest);
    Ok(artifact)
}

pub fn import_video(app: &AppHandle, state: &Arc<AppState>, meeting_group_id: &str, path: &str) -> AppResult<Artifact> {
    let source = PathBuf::from(path);
    let ext = ext_of(&source);
    if !IMPORT_VIDEO_EXTS.contains(&ext.as_str()) {
        if VIDEO_EXTS.contains(&ext.as_str()) {
            return Err(AppError::Message(
                "BellaNote can import MP4 video. Convert this file to MP4, or export audio instead.".into(),
            ));
        }
        return Err(AppError::Message(
            "BellaNote needs an MP4 video file.".into(),
        ));
    }
    if !source.is_file() {
        return Err(AppError::Message("Couldn’t read the file.".into()));
    }

    let id = Uuid::new_v4().to_string();
    let video_dest = copy_video_into_library(app, &id, &source)?;
    let filename = source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video")
        .to_string();
    let artifact = Artifact {
        id: id.clone(),
        meeting_group_id: meeting_group_id.to_string(),
        title: title_from_path(&source),
        source_type: "video_upload".into(),
        status: "queued".into(),
        has_audio: true,
        original_filename: filename,
        original_path: source.to_string_lossy().into_owned(),
        error_message: String::new(),
        transcript: String::new(),
        segments_json: "[]".into(),
        duration_ms: 0,
        created_at: Utc::now().to_rfc3339(),
        whisper_model: String::new(),
    };
    state.db.insert_artifact(&artifact)?;
    spawn_video_import(app.clone(), Arc::clone(state), id, video_dest);
    Ok(artifact)
}

pub fn retry_artifact(app: &AppHandle, state: &Arc<AppState>, id: &str) -> AppResult<Artifact> {
    let artifact = state.db.get_artifact(id)?;
    if artifact.status == "queued" || artifact.status == "transcribing" || artifact.status == "recording" {
        return Err(AppError::Message("This file is already importing.".into()));
    }
    let empty_ready = artifact.status == "ready"
        && artifact.has_audio
        && artifact.transcript.trim().is_empty();
    if artifact.status != "failed" && !empty_ready {
        return Err(AppError::Message("Only failed imports can be retried.".into()));
    }
    if !artifact.has_audio {
        return Err(AppError::Message("This file has no audio to transcribe.".into()));
    }
    let path = find_audio_path(app, id).ok_or_else(|| {
        AppError::Message("The original audio is missing. Remove this file and add it again.".into())
    })?;
    state.db.set_artifact_status(id, "queued", "")?;
    let _ = app.emit("artifact-updated", id);
    spawn_transcribe(app.clone(), Arc::clone(state), id.to_string(), path);
    state.db.get_artifact(id)
}

pub fn import_transcript_file(
    app: &AppHandle,
    state: &Arc<AppState>,
    meeting_group_id: &str,
    path: &str,
) -> AppResult<Artifact> {
    let source = PathBuf::from(path);
    let ext = ext_of(&source);
    if AUDIO_EXTS.contains(&ext.as_str()) {
        return Err(AppError::Message(
            "That looks like audio. Use Add audio instead.".into(),
        ));
    }
    if !TRANSCRIPT_EXTS.contains(&ext.as_str()) {
        return Err(AppError::Message(
            "Import a .vtt, .srt, or .txt transcript.".into(),
        ));
    }
    let raw = std::fs::read_to_string(&source)
        .map_err(|e| AppError::Message(format!("Could not read file: {e}")))?;
    let (segs, full) = parse_transcript_file(&source, &raw)?;
    let id = Uuid::new_v4().to_string();
    let dest = copy_into_library(app, &id, &source)?;
    let _ = dest;
    let duration_ms = segs.iter().map(|s| s.end_ms).max().unwrap_or(0);
    let segments_json = serde_json::to_string(&segs).unwrap_or_else(|_| "[]".into());
    let filename = source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("transcript")
        .to_string();
    let artifact = Artifact {
        id,
        meeting_group_id: meeting_group_id.to_string(),
        title: title_from_path(&source),
        source_type: "transcript_import".into(),
        status: "ready".into(),
        has_audio: false,
        original_filename: filename,
        original_path: source.to_string_lossy().into_owned(),
        error_message: String::new(),
        transcript: full,
        segments_json,
        duration_ms,
        created_at: Utc::now().to_rfc3339(),
        whisper_model: "imported".into(),
    };
    state.db.insert_artifact(&artifact)?;
    Ok(artifact)
}

pub fn delete_artifact_files(app: &AppHandle, id: &str) {
    if let Ok(dir) = paths::artifact_dir(app, id) {
        let _ = std::fs::remove_dir_all(dir);
    }
}

pub fn find_audio_path(app: &AppHandle, id: &str) -> Option<PathBuf> {
    let dir = paths::artifact_dir(app, id).ok()?;
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("source."))
            && AUDIO_EXTS.contains(&ext_of(p).as_str())
    })
}

pub fn find_video_path(app: &AppHandle, id: &str) -> Option<PathBuf> {
    let dir = paths::artifact_dir(app, id).ok()?;
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("video."))
            && VIDEO_EXTS.contains(&ext_of(p).as_str())
    })
}

/// Copy BellaNote's library audio to a user-chosen path. Never deletes or moves the library file.
pub fn export_artifact_audio(app: &AppHandle, id: &str, dest: &str) -> AppResult<()> {
    let source = find_audio_path(app, id).ok_or_else(|| {
        AppError::Message("This file has no audio to export.".into())
    })?;
    let dest_path = PathBuf::from(dest);
    if dest_path.as_os_str().is_empty() {
        return Err(AppError::Message("Choose where to save the audio.".into()));
    }
    if let Ok(src) = source.canonicalize() {
        if let Ok(dst) = dest_path.canonicalize() {
            if src == dst {
                return Ok(());
            }
        }
    }
    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return Err(AppError::Message("That folder doesn’t exist.".into()));
        }
    }
    std::fs::copy(&source, &dest_path).map_err(|e| {
        AppError::Message(format!("Could not export the audio file: {e}"))
    })?;
    Ok(())
}

fn format_export_timestamp(ms: i64) -> String {
    let total = ms.max(0) / 1000;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn transcript_export_body(artifact: &Artifact) -> AppResult<String> {
    let plain = artifact.transcript.trim();
    if plain.is_empty() {
        return Err(AppError::Message("This file has no transcript to export.".into()));
    }

    #[derive(serde::Deserialize)]
    struct Seg {
        text: String,
        start_ms: i64,
    }

    if let Ok(segs) = serde_json::from_str::<Vec<Seg>>(&artifact.segments_json) {
        let lines: Vec<String> = segs
            .into_iter()
            .filter_map(|seg| {
                let text = seg.text.trim();
                if text.is_empty() {
                    None
                } else {
                    Some(format!("[{}] {}", format_export_timestamp(seg.start_ms), text))
                }
            })
            .collect();
        if !lines.is_empty() {
            return Ok(lines.join("\n"));
        }
    }

    Ok(plain.to_string())
}

/// Write the artifact transcript to a user-chosen `.txt` path.
pub fn export_artifact_transcript(
    state: &Arc<AppState>,
    id: &str,
    dest: &str,
) -> AppResult<()> {
    let artifact = state.db.get_artifact(id)?;
    let body = transcript_export_body(&artifact)?;
    let dest_path = PathBuf::from(dest);
    if dest_path.as_os_str().is_empty() {
        return Err(AppError::Message("Choose where to save the transcript.".into()));
    }
    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return Err(AppError::Message("That folder doesn’t exist.".into()));
        }
    }
    let mut out = body;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(&dest_path, out.as_bytes()).map_err(|e| {
        AppError::Message(format!("Could not export the transcript: {e}"))
    })?;
    Ok(())
}

pub(crate) fn spawn_transcribe(app: AppHandle, state: Arc<AppState>, id: String, audio_path: PathBuf) {
    std::thread::spawn(move || {
        let result = (|| {
            let mut slot = state
                .transcriber
                .lock()
                .map_err(|_| anyhow::anyhow!("transcriber lock poisoned"))?;
            if slot.is_none() {
                *slot = Some(Transcriber::new(&app)?);
            }
            let _ = state.db.set_artifact_status(&id, "transcribing", "");
            let _ = app.emit("artifact-updated", &id);
            slot.as_ref().unwrap().transcribe_path(&audio_path)
        })();
        finish_transcribe(&app, &state, &id, result);
    });
}

/// Extract audio from a stored video, then run the normal Whisper path on the extract.
pub(crate) fn spawn_video_import(app: AppHandle, state: Arc<AppState>, id: String, video_path: PathBuf) {
    std::thread::spawn(move || {
        let audio_path = (|| -> anyhow::Result<PathBuf> {
            let mut slot = state
                .transcriber
                .lock()
                .map_err(|_| anyhow::anyhow!("transcriber lock poisoned"))?;
            if slot.is_none() {
                *slot = Some(Transcriber::new(&app)?);
            }
            let transcriber = slot.as_ref().unwrap();
            let dir = video_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("video path has no parent"))?;
            let m4a = dir.join("source.m4a");
            let wav = dir.join("source.wav");
            match transcriber.extract_audio(&video_path, &m4a) {
                Ok(()) => Ok(m4a),
                Err(first) => {
                    let _ = std::fs::remove_file(&m4a);
                    transcriber
                        .extract_audio(&video_path, &wav)
                        .map_err(|second| {
                            anyhow::anyhow!("extract audio failed ({first}); wav fallback: {second}")
                        })?;
                    Ok(wav)
                }
            }
        })();

        match audio_path {
            Ok(path) => {
                let result = (|| {
                    let mut slot = state
                        .transcriber
                        .lock()
                        .map_err(|_| anyhow::anyhow!("transcriber lock poisoned"))?;
                    if slot.is_none() {
                        *slot = Some(Transcriber::new(&app)?);
                    }
                    let _ = state.db.set_artifact_status(&id, "transcribing", "");
                    let _ = app.emit("artifact-updated", &id);
                    slot.as_ref().unwrap().transcribe_path(&path)
                })();
                finish_transcribe(&app, &state, &id, result);
            }
            Err(err) => {
                let _ = state
                    .db
                    .set_artifact_status(&id, "failed", &human_fail_message(&err));
                let _ = app.emit("artifact-updated", &id);
            }
        }
    });
}

fn finish_transcribe(
    app: &AppHandle,
    state: &Arc<AppState>,
    id: &str,
    result: anyhow::Result<Vec<crate::transcribe::TranscriptSegment>>,
) {
    match result {
        Ok(segments) if segments.is_empty() => {
            let _ = state.db.set_artifact_status(
                id,
                "failed",
                "Transcription didn’t finish. The recording is saved — you can try again.",
            );
        }
        Ok(segments) => {
            let transcript = segments
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if transcript.trim().is_empty() {
                let _ = state.db.set_artifact_status(
                    id,
                    "failed",
                    "Transcription didn’t finish. The recording is saved — you can try again.",
                );
            } else {
                let duration_ms = segments.iter().map(|s| s.end_ms).max().unwrap_or(0);
                let json = serde_json::to_string(&segments).unwrap_or_else(|_| "[]".into());
                let _ = state.db.set_artifact_transcript(
                    id,
                    &transcript,
                    &json,
                    duration_ms,
                    &crate::transcribe::whisper_model(),
                );
            }
        }
        Err(err) => {
            let _ = state
                .db
                .set_artifact_status(id, "failed", &human_fail_message(&err));
        }
    }
    let _ = app.emit("artifact-updated", id);
}

fn human_fail_message(err: &anyhow::Error) -> String {
    let text = err.to_string();
    let lower = text.to_lowercase();
    if lower.contains("spawn")
        || lower.contains("venv")
        || lower.contains("worker not found")
        || lower.contains("echo_python")
        || lower.contains("team id")
        || lower.contains("python shared library")
        || lower.contains("bundled transcription worker")
    {
        return "Transcription isn’t available on this Mac. You can try again.".into();
    }
    if lower.contains("no such file") || lower.contains("couldn’t read") || lower.contains("could not")
    {
        return "BellaNote couldn’t read this audio file.".into();
    }
    if lower.contains("extract") || lower.contains("no audio track") {
        return "BellaNote couldn’t pull audio from this video. You can try again.".into();
    }
    "This file couldn’t be processed. You can try again.".into()
}

