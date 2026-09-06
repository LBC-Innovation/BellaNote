//! Persistent faster-whisper worker. This POC uses `small.en` for uploaded files.

use anyhow::{Context, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TranscriptSegment {
    pub text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    #[serde(default)]
    pub avg_logprob: Option<f64>,
    #[serde(default)]
    pub no_speech_prob: Option<f64>,
}

struct TranscribeProcess {
    child: Child,
    stdin: BufWriter<std::process::ChildStdin>,
    stdout: BufReader<std::process::ChildStdout>,
}

impl Drop for TranscribeProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Transcriber {
    inner: Arc<Mutex<TranscribeProcess>>,
}

fn host_triple() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "unknown"
    }
}

fn sidecar_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ECHO_TRANSCRIBE_SIDECAR") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    if cfg!(debug_assertions) {
        return None;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let next_to_app = dir.join("transcribe-worker");
            if next_to_app.is_file() {
                return Some(next_to_app);
            }
        }
    }
    let packaged = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(format!("transcribe-worker-{}", host_triple()));
    if packaged.is_file() {
        return Some(packaged);
    }
    None
}

fn worker_script_path() -> PathBuf {
    if let Ok(p) = std::env::var("ECHO_TRANSCRIBE_SCRIPT") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/transcribe_worker.py")
}

fn default_python() -> PathBuf {
    if let Ok(p) = std::env::var("ECHO_PYTHON") {
        return PathBuf::from(p);
    }
    let venv = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.venv/bin/python3");
    if venv.is_file() {
        return venv;
    }
    PathBuf::from("python3")
}

pub fn whisper_model() -> String {
    std::env::var("WHISPER_MODEL").unwrap_or_else(|_| "small.en".to_string())
}

fn whisper_download_root(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_data_dir().ok()?.join("whisper-models");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn apply_worker_args(cmd: &mut Command, app: &AppHandle) {
    cmd.arg("--model").arg(whisper_model());
    if let Some(root) = whisper_download_root(app) {
        cmd.arg("--download-root").arg(root);
    }
}

fn spawn_worker(app: &AppHandle) -> Result<Child> {
    let mut cmd = if let Some(sidecar) = sidecar_path() {
        Command::new(sidecar)
    } else if cfg!(debug_assertions) {
        let script = worker_script_path();
        if !script.is_file() {
            anyhow::bail!("transcribe worker not found at {:?}", script);
        }
        let mut cmd = Command::new(default_python());
        cmd.arg(script);
        cmd
    } else {
        anyhow::bail!(
            "The bundled transcription worker is missing from this build. Rebuild with npm run tauri:build."
        );
    };
    apply_worker_args(&mut cmd, app);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to start the transcription worker")
}

impl Transcriber {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let mut child = spawn_worker(app)?;
        let stdout = BufReader::new(child.stdout.take().context("no stdout")?);
        let stdin = BufWriter::new(child.stdin.take().context("no stdin")?);
        let mut proc = TranscribeProcess {
            child,
            stdin,
            stdout,
        };

        let mut line = String::new();
        proc.stdout
            .read_line(&mut line)
            .context("read worker ready line")?;
        let v: serde_json::Value =
            serde_json::from_str(line.trim()).with_context(|| format!("bad JSON: {line:?}"))?;
        if let Some(err) = v.get("error") {
            anyhow::bail!("faster-whisper worker: {}", err);
        }
        if v.get("status").and_then(|s| s.as_str()) != Some("ready") {
            anyhow::bail!("unexpected worker handshake: {line}");
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(proc)),
        })
    }

    pub fn transcribe_path(&self, path: &Path) -> Result<Vec<TranscriptSegment>> {
        let path_str = path
            .to_str()
            .with_context(|| format!("path must be UTF-8: {path:?}"))?;
        let line_out = {
            let mut g = self.inner.lock().expect("transcribe mutex poisoned");
            let req = serde_json::json!({ "wav_path": path_str });
            writeln!(g.stdin, "{req}")?;
            g.stdin.flush()?;
            let mut line = String::new();
            g.stdout
                .read_line(&mut line)
                .context("read worker response")?;
            line
        };

        let v: serde_json::Value = serde_json::from_str(line_out.trim())
            .with_context(|| format!("bad JSON: {line_out:?}"))?;
        if let Some(err) = v.get("error") {
            let detail = v.get("detail").unwrap_or(err);
            anyhow::bail!("faster-whisper: {err} — {detail}");
        }

        let segments: Vec<TranscriptSegment> =
            serde_json::from_value(v.get("segments").cloned().unwrap_or(serde_json::json!([])))
                .context("parse segments")?;
        Ok(segments
            .into_iter()
            .filter(|s| !s.text.trim().is_empty())
            .collect())
    }

    pub fn transcribe_samples(&self, samples: &[f32]) -> Result<Vec<TranscriptSegment>> {
        let tmp = tempfile::Builder::new()
            .suffix(".wav")
            .tempfile()
            .context("temp wav")?;
        write_wav_f32(tmp.path(), samples).context("write temp wav")?;
        let segments = self.transcribe_path(tmp.path())?;
        drop(tmp);
        Ok(segments)
    }
}

fn write_wav_f32(path: &Path, samples: &[f32]) -> Result<()> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut w = WavWriter::create(path, spec).context("WavWriter::create")?;
    for &s in samples {
        w.write_sample(s)?;
    }
    w.finalize().context("finalize wav")?;
    Ok(())
}
