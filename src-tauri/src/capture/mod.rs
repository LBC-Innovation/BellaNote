pub mod mic;
pub mod mixer;
pub mod spectrum;
pub mod wav;

#[cfg(target_os = "macos")]
pub mod system_macos;
#[cfg(target_os = "windows")]
pub mod system_windows;

use crate::capture::mic::{MicCaptureSession, SampleSink};
use crate::capture::mixer::SystemMicMixer;
use crate::capture::wav::MixOutput;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

#[cfg(target_os = "macos")]
pub use system_macos::SystemAudioSession;
#[cfg(target_os = "windows")]
pub use system_windows::SystemAudioSession;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub struct SystemAudioSession;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl SystemAudioSession {
    pub async fn new(_mixer: std::sync::Arc<SystemMicMixer>) -> Result<Self> {
        Err(anyhow::anyhow!(
            "System audio capture is not available on this platform."
        ))
    }

    pub fn start(&self) -> Result<()> {
        Err(anyhow::anyhow!(
            "System audio capture is not available on this platform."
        ))
    }

    pub fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    Voice,
    SystemAudio,
}

pub enum AnyCaptureSession {
    Voice {
        mic: MicCaptureSession,
    },
    System {
        mic: MicCaptureSession,
        system: SystemAudioSession,
        mixer_running: Arc<AtomicBool>,
        mixer_join: std::sync::Mutex<Option<JoinHandle<()>>>,
    },
}

impl AnyCaptureSession {
    pub fn start_voice(output: MixOutput) -> Result<Self> {
        let sink: SampleSink = Arc::new(move |samples: &[f32]| output.emit_mono_16k(samples));
        Ok(Self::Voice {
            mic: MicCaptureSession::new(sink, crate::capture::wav::SAMPLE_RATE)?,
        })
    }

    pub async fn start_system(output: MixOutput) -> Result<Self> {
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = output;
            return Err(anyhow::anyhow!(
                "System audio capture is not available on this platform."
            ));
        }
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let mixer = SystemMicMixer::new(output);
            let mixer_running = Arc::new(AtomicBool::new(true));
            let mixer_join =
                SystemMicMixer::spawn_mixer_thread(mixer.clone(), mixer_running.clone());
            let mixer_for_mic = mixer.clone();
            let mic_sink: SampleSink =
                Arc::new(move |samples: &[f32]| mixer_for_mic.push_microphone(samples));
            let mic = match MicCaptureSession::new(mic_sink, crate::capture::wav::PLAYBACK_RATE) {
                Ok(m) => m,
                Err(e) => {
                    mixer_running.store(false, Ordering::Release);
                    let _ = mixer_join.join();
                    return Err(e);
                }
            };
            let system = match SystemAudioSession::new(mixer).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = mic.stop();
                    mixer_running.store(false, Ordering::Release);
                    let _ = mixer_join.join();
                    return Err(e);
                }
            };
            if let Err(e) = system.start() {
                let _ = mic.stop();
                let _ = system.stop();
                mixer_running.store(false, Ordering::Release);
                let _ = mixer_join.join();
                return Err(e);
            }
            Ok(Self::System {
                mic,
                system,
                mixer_running,
                mixer_join: std::sync::Mutex::new(Some(mixer_join)),
            })
        }
    }

    pub fn stop(&self) -> Result<()> {
        match self {
            Self::Voice { mic } => mic.stop(),
            Self::System {
                mic,
                system,
                mixer_running,
                mixer_join,
            } => {
                let sys_err = system.stop();
                let mic_err = mic.stop();
                mixer_running.store(false, Ordering::Release);
                if let Ok(mut slot) = mixer_join.lock() {
                    if let Some(h) = slot.take() {
                        let _ = h.join();
                    }
                }
                sys_err.and(mic_err)
            }
        }
    }
}
