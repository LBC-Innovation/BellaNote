use crate::capture::spectrum::SpectrumBuf;
use crate::capture::wav::{
    create_wav_writer, system_wav_spec, voice_wav_spec, Downmix48kTo16k, MixOutput,
    RecordingWavWriter, SAMPLE_RATE,
};
use crate::capture::{AnyCaptureSession, CaptureMode};
use crate::db::Artifact;
use crate::error::{AppError, AppResult};
use crate::paths;
use crate::state::AppState;
use crate::transcribe::TranscriptSegment;
use chrono::Utc;
use ringbuf::HeapRb;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::watch;
use tokio::time::interval;
use uuid::Uuid;

pub struct RecordingRuntime {
    pub session: Mutex<Option<RecordingSession>>,
    pub rms_tx: watch::Sender<f32>,
    pub rms_rx: watch::Receiver<f32>,
    pub spectrum: Arc<Mutex<SpectrumBuf>>,
}

impl RecordingRuntime {
    pub fn new() -> Self {
        let (rms_tx, rms_rx) = watch::channel(0.0f32);
        Self {
            session: Mutex::new(None),
            rms_tx,
            rms_rx,
            spectrum: Arc::new(Mutex::new(SpectrumBuf::new())),
        }
    }
}

pub struct RecordingSession {
    pub artifact_id: String,
    pub meeting_group_id: String,
    pub source: String,
    pub started_at_unix_ms: i64,
    capture: AnyCaptureSession,
    wav: Arc<Mutex<Option<RecordingWavWriter>>>,
    in_progress_path: PathBuf,
    source_path: PathBuf,
    cancel: Arc<AtomicBool>,
    whisper_join: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatus {
    pub active: bool,
    pub artifact_id: Option<String>,
    pub meeting_group_id: Option<String>,
    pub source: Option<String>,
    pub started_at_unix_ms: Option<i64>,
    pub input_label: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordingCapabilities {
    pub microphone: bool,
    pub system_audio: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingResult {
    pub artifact: Artifact,
    pub input_label: String,
}

pub fn capabilities() -> RecordingCapabilities {
    RecordingCapabilities {
        microphone: true,
        system_audio: cfg!(any(target_os = "macos", target_os = "windows")),
    }
}

pub fn status(state: &AppState) -> RecordingStatus {
    let guard = state.recording.session.lock().unwrap();
    match guard.as_ref() {
        Some(s) => RecordingStatus {
            active: true,
            artifact_id: Some(s.artifact_id.clone()),
            meeting_group_id: Some(s.meeting_group_id.clone()),
            source: Some(s.source.clone()),
            started_at_unix_ms: Some(s.started_at_unix_ms),
            input_label: None,
        },
        None => RecordingStatus {
            active: false,
            artifact_id: None,
            meeting_group_id: None,
            source: None,
            started_at_unix_ms: None,
            input_label: None,
        },
    }
}

pub fn rms(state: &AppState) -> f32 {
    *state.recording.rms_rx.borrow()
}

pub fn spectrum(state: &AppState) -> Vec<f32> {
    state
        .recording
        .spectrum
        .lock()
        .map(|buf| buf.bands(SAMPLE_RATE as f32))
        .unwrap_or_else(|_| vec![0.0; crate::capture::spectrum::SPECTRUM_BANDS])
}

fn parse_mode(source: &str) -> AppResult<CaptureMode> {
    match source {
        "voice" | "microphone" => Ok(CaptureMode::Voice),
        "system" | "system_audio" => Ok(CaptureMode::SystemAudio),
        _ => Err(AppError::Message(
            "Choose Microphone or System audio.".into(),
        )),
    }
}

fn source_type(mode: CaptureMode) -> &'static str {
    match mode {
        CaptureMode::Voice => "voice",
        CaptureMode::SystemAudio => "system",
    }
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub async fn start_recording(
    app: AppHandle,
    state: Arc<AppState>,
    meeting_group_id: String,
    source: String,
) -> AppResult<StartRecordingResult> {
    {
        let session = state.recording.session.lock().unwrap();
        if session.is_some() {
            return Err(AppError::Message(
                "A recording is already in progress.".into(),
            ));
        }
    }

    let mode = parse_mode(&source)?;
    if mode == CaptureMode::SystemAudio && !cfg!(any(target_os = "macos", target_os = "windows")) {
        return Err(AppError::Message(
            "System audio capture is not available on this platform.".into(),
        ));
    }

    let _ = state.db.list_artifacts(&meeting_group_id)?;

    let id = Uuid::new_v4().to_string();
    let source_ty = source_type(mode);
    let title = match mode {
        CaptureMode::Voice => "Microphone recording",
        CaptureMode::SystemAudio => "System audio recording",
    };
    let artifact = Artifact {
        id: id.clone(),
        meeting_group_id: meeting_group_id.clone(),
        title: title.into(),
        source_type: source_ty.into(),
        status: "recording".into(),
        has_audio: true,
        original_filename: "recording.wav".into(),
        original_path: String::new(),
        error_message: String::new(),
        transcript: String::new(),
        segments_json: "[]".into(),
        duration_ms: 0,
        created_at: Utc::now().to_rfc3339(),
        whisper_model: String::new(),
    };
    state.db.insert_artifact(&artifact)?;
    let _ = app.emit("artifact-updated", &id);

    let dir = paths::artifact_dir(&app, &id).map_err(AppError::from)?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Message(e.to_string()))?;
    let in_progress_path = dir.join("in_progress.wav");
    let source_path = dir.join("source.wav");
    if in_progress_path.exists() {
        let _ = std::fs::remove_file(&in_progress_path);
    }

    let spec = match mode {
        CaptureMode::Voice => voice_wav_spec(),
        CaptureMode::SystemAudio => system_wav_spec(),
    };
    let wav_writer = match create_wav_writer(&in_progress_path, spec) {
        Ok(w) => w,
        Err(e) => {
            let _ =
                state
                    .db
                    .set_artifact_status(&id, "failed", "Could not start the recording file.");
            let _ = app.emit("artifact-updated", &id);
            return Err(AppError::Message(e));
        }
    };
    let wav = Arc::new(Mutex::new(Some(wav_writer)));

    let rb = HeapRb::<f32>::new(SAMPLE_RATE as usize * 60);
    let (producer, mut consumer) = rb.split();
    let producer = Arc::new(Mutex::new(producer));
    let output = MixOutput {
        producer,
        rms_sender: state.recording.rms_tx.clone(),
        spectrum: state.recording.spectrum.clone(),
        wav: wav.clone(),
        whisper_downmix: Arc::new(Mutex::new(Downmix48kTo16k::default())),
    };

    let capture = match mode {
        CaptureMode::Voice => AnyCaptureSession::start_voice(output),
        CaptureMode::SystemAudio => AnyCaptureSession::start_system(output).await,
    };
    let capture = match capture {
        Ok(c) => c,
        Err(e) => {
            let _ =
                state
                    .db
                    .set_artifact_status(&id, "failed", &human_capture_error(&e.to_string()));
            let _ = app.emit("artifact-updated", &id);
            return Err(AppError::Message(human_capture_error(&e.to_string())));
        }
    };

    let cancel = Arc::new(AtomicBool::new(false));
    let (tick_secs, chunk_samples, min_chunk_samples) = match mode {
        CaptureMode::Voice => (3u64, SAMPLE_RATE as usize * 3, 8_000),
        CaptureMode::SystemAudio => (10u64, SAMPLE_RATE as usize * 10, 4_000),
    };

    let whisper_state = Arc::clone(&state);
    let whisper_app = app.clone();
    let whisper_id = id.clone();
    let whisper_cancel = cancel.clone();
    let join = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(tick_secs));
        let mut all_segments: Vec<TranscriptSegment> = Vec::new();
        let mut stream_offset_samples: u64 = 0;
        loop {
            ticker.tick().await;
            let cancelled = whisper_cancel.load(Ordering::Acquire);
            let mut chunk: Vec<f32> = Vec::with_capacity(chunk_samples);
            while let Some(s) = consumer.pop() {
                chunk.push(s);
                if chunk.len() >= chunk_samples {
                    break;
                }
            }
            let n = chunk.len();
            let enough = if cancelled {
                n > 1_600
            } else {
                n >= min_chunk_samples
            };
            if enough {
                let chunk_start_ms = (stream_offset_samples * 1000) / u64::from(SAMPLE_RATE);
                stream_offset_samples += n as u64;
                let samples = chunk;
                let transcriber_result = tokio::task::spawn_blocking({
                    let whisper_state = Arc::clone(&whisper_state);
                    let app = whisper_app.clone();
                    move || {
                        let mut slot = whisper_state
                            .transcriber
                            .lock()
                            .map_err(|_| anyhow::anyhow!("transcriber lock poisoned"))?;
                        if slot.is_none() {
                            *slot = Some(crate::transcribe::Transcriber::new(&app)?);
                        }
                        slot.as_ref().unwrap().transcribe_samples(&samples)
                    }
                })
                .await;
                if let Ok(Ok(mut new_segments)) = transcriber_result {
                    let off = chunk_start_ms as i64;
                    for seg in &mut new_segments {
                        seg.start_ms += off;
                        seg.end_ms += off;
                    }
                    all_segments.extend(new_segments);
                    let full_text: String = all_segments
                        .iter()
                        .map(|s| s.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let segments_json = serde_json::to_string(&all_segments).unwrap_or_default();
                    let duration_ms = all_segments.iter().map(|s| s.end_ms).max().unwrap_or(
                        ((stream_offset_samples * 1000) / u64::from(SAMPLE_RATE)) as i64,
                    );
                    let _ = whisper_state.db.set_artifact_live_transcript(
                        &whisper_id,
                        &full_text,
                        &segments_json,
                        duration_ms,
                    );
                    let _ = whisper_app.emit("artifact-updated", &whisper_id);
                }
            } else {
                stream_offset_samples += n as u64;
            }
            if cancelled {
                let _ = whisper_state
                    .db
                    .set_artifact_status(&whisper_id, "transcribing", "");
                let _ = whisper_app.emit("artifact-updated", &whisper_id);
                break;
            }
        }
        let duration_ms = ((stream_offset_samples * 1000) / u64::from(SAMPLE_RATE)) as i64;
        let full_text: String = all_segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let segments_json = serde_json::to_string(&all_segments).unwrap_or_else(|_| "[]".into());
        let _ = whisper_state.db.set_artifact_transcript(
            &whisper_id,
            &full_text,
            &segments_json,
            duration_ms.max(
                all_segments
                    .iter()
                    .map(|s| s.end_ms)
                    .max()
                    .unwrap_or(duration_ms),
            ),
            &crate::transcribe::whisper_model(),
        );
        let _ = whisper_app.emit("artifact-updated", &whisper_id);
    });

    let input_label = match mode {
        CaptureMode::Voice => {
            crate::capture::mic::default_input_device_name().unwrap_or_else(|_| "Microphone".into())
        }
        CaptureMode::SystemAudio => "System + mic (what you hear and what you say)".to_string(),
    };

    let session = RecordingSession {
        artifact_id: id,
        meeting_group_id,
        source: source_ty.to_string(),
        started_at_unix_ms: now_unix_ms(),
        capture,
        wav,
        in_progress_path,
        source_path,
        cancel,
        whisper_join: Mutex::new(Some(join)),
    };
    *state.recording.session.lock().unwrap() = Some(session);

    Ok(StartRecordingResult {
        artifact,
        input_label,
    })
}

pub async fn stop_recording(app: AppHandle, state: Arc<AppState>) -> AppResult<Artifact> {
    let session = state
        .recording
        .session
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AppError::Message("No recording is in progress.".into()))?;

    session.cancel.store(true, Ordering::Release);
    let stop_err = session.capture.stop();
    finalize_wav(&session);

    if let Some(join) = session.whisper_join.lock().unwrap().take() {
        // Trailing Whisper keeps running; stop returns immediately.
        drop(join);
    }

    if let Err(e) = stop_err {
        let _ = state.db.set_artifact_status(
            &session.artifact_id,
            "failed",
            &human_capture_error(&e.to_string()),
        );
        let _ = app.emit("artifact-updated", &session.artifact_id);
        return Err(AppError::Message(human_capture_error(&e.to_string())));
    }

    let _ = state.recording.rms_tx.send(0.0);
    if let Ok(mut spectrum) = state.recording.spectrum.lock() {
        spectrum.clear();
    }
    state.db.get_artifact(&session.artifact_id)
}

pub async fn abort_if_artifact(app: &AppHandle, state: &Arc<AppState>, artifact_id: &str) {
    let matches = {
        let g = state.recording.session.lock().unwrap();
        g.as_ref().is_some_and(|s| s.artifact_id == artifact_id)
    };
    if matches {
        let _ = stop_recording(app.clone(), Arc::clone(state)).await;
    }
}

pub async fn abort_if_group(app: &AppHandle, state: &Arc<AppState>, meeting_group_id: &str) {
    let matches = {
        let g = state.recording.session.lock().unwrap();
        g.as_ref()
            .is_some_and(|s| s.meeting_group_id == meeting_group_id)
    };
    if matches {
        let _ = stop_recording(app.clone(), Arc::clone(state)).await;
    }
}

fn finalize_wav(session: &RecordingSession) {
    if let Ok(mut slot) = session.wav.lock() {
        if let Some(writer) = slot.take() {
            let _ = writer.finalize();
        }
    }
    if session.in_progress_path.is_file() {
        let _ = std::fs::rename(&session.in_progress_path, &session.source_path);
    }
}

fn human_capture_error(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains("no default input") || lower.contains("microphone") {
        #[cfg(target_os = "windows")]
        {
            return "BellaNote couldn’t use the microphone. Check Settings → Privacy & security → Microphone.".into();
        }
        #[cfg(not(target_os = "windows"))]
        {
            return "BellaNote couldn’t use the microphone. Check System Settings → Privacy & Security.".into();
        }
    }
    if lower.contains("wasapi")
        || lower.contains("loopback")
        || lower.contains("playback device")
        || lower.contains("render")
    {
        return "BellaNote couldn’t capture system audio. Check that a playback device is available, then try again.".into();
    }
    if lower.contains("screen") || lower.contains("shareable") || lower.contains("display") {
        return "BellaNote needs Screen Recording permission to capture system audio. Check System Settings → Privacy & Security.".into();
    }
    "BellaNote couldn’t start recording.".into()
}
