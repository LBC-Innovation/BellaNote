//! Placeholder for WASAPI loopback. Microphone capture already works via `cpal`.

use anyhow::Result;

pub struct SystemAudioSession;

impl SystemAudioSession {
    pub async fn new(_mixer: std::sync::Arc<crate::capture::mixer::SystemMicMixer>) -> Result<Self> {
        Err(anyhow::anyhow!(
            "System audio capture is not available on Windows yet."
        ))
    }

    pub fn start(&self) -> Result<()> {
        Err(anyhow::anyhow!(
            "System audio capture is not available on Windows yet."
        ))
    }

    pub fn stop(&self) -> Result<()> {
        Ok(())
    }
}
