//! Persistent faster-whisper worker. This POC uses `small.en` for uploaded files.

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

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

fn worker_script_path() -> PathBuf {
    if let Ok(p) = std::env::var("ECHO_TRANSCRIBE_SCRIPT") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/transcribe_worker.py")
}

fn whisper_model_arg() -> String {
    std::env::var("WHISPER_MODEL").unwrap_or_else(|_| "small.en".to_string())
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

impl Transcriber {
    pub fn new() -> Result<Self> {
        let python = default_python();
        let script = worker_script_path();
        let model = whisper_model_arg();
        if !script.is_file() {
            anyhow::bail!("transcribe worker not found at {:?}", script);
        }

        let mut child = Command::new(&python)
            .arg(&script)
            .arg("--model")
            .arg(&model)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| {
                format!(
                    "failed to spawn {:?} (create .venv or set ECHO_PYTHON)",
                    python
                )
            })?;

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
}
