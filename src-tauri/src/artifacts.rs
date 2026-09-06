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

pub fn import_audio(app: &AppHandle, state: &Arc<AppState>, meeting_group_id: &str, path: &str) -> AppResult<Artifact> {
    let source = PathBuf::from(path);
    let ext = ext_of(&source);
    if VIDEO_EXTS.contains(&ext.as_str()) {
        return Err(AppError::Message(
            "Video files are not supported yet. Please add an audio file.".into(),
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

pub fn retry_artifact(app: &AppHandle, state: &Arc<AppState>, id: &str) -> AppResult<Artifact> {
    let artifact = state.db.get_artifact(id)?;
    if artifact.status == "queued" || artifact.status == "transcribing" {
        return Err(AppError::Message("This file is already importing.".into()));
    }
    if artifact.status != "failed" {
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

fn spawn_transcribe(app: AppHandle, state: Arc<AppState>, id: String, audio_path: PathBuf) {
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
        match result {
            Ok(segments) => {
                let transcript = segments
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let duration_ms = segments.iter().map(|s| s.end_ms).max().unwrap_or(0);
                let json = serde_json::to_string(&segments).unwrap_or_else(|_| "[]".into());
                let _ = state.db.set_artifact_transcript(
                    &id,
                    &transcript,
                    &json,
                    duration_ms,
                    &crate::transcribe::whisper_model(),
                );
            }
            Err(err) => {
                let _ = state
                    .db
                    .set_artifact_status(&id, "failed", &human_fail_message(&err));
            }
        }
        let _ = app.emit("artifact-updated", &id);
    });
}

fn human_fail_message(err: &anyhow::Error) -> String {
    let text = err.to_string();
    let lower = text.to_lowercase();
    if lower.contains("spawn")
        || lower.contains("venv")
        || lower.contains("worker not found")
        || lower.contains("echo_python")
    {
        return "Transcription isn’t available on this Mac. You can try again.".into();
    }
    if lower.contains("no such file") || lower.contains("couldn’t read") || lower.contains("could not")
    {
        return "BellaNote couldn’t read this audio file.".into();
    }
    "This file couldn’t be processed. You can try again.".into()
}

