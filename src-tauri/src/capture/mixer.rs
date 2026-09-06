//! Clocked mixer for system audio + microphone.
//! Pairing samples inside capture callbacks misaligns two clocks (jitter / pumping).
//! System stays stereo; the mic is added equally to both sides (center).

use crate::capture::wav::{MixOutput, PLAYBACK_RATE};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// 10 ms @ 48 kHz.
pub const MIX_CHUNK_FRAMES: usize = PLAYBACK_RATE as usize / 100;
const MIX_FRAME: Duration = Duration::from_millis(10);
/// Drop oldest frames if a source runs more than 100 ms ahead.
const MAX_QUEUE_FRAMES: usize = 4_800;
/// Mix each source near −6 dBFS before summing.
const SOURCE_GAIN: f32 = 0.5;

pub struct SystemMicMixer {
    output: MixOutput,
    sys_pending: Mutex<VecDeque<[f32; 2]>>,
    mic_pending: Mutex<VecDeque<f32>>,
}

impl SystemMicMixer {
    pub fn new(output: MixOutput) -> Arc<Self> {
        Arc::new(Self {
            output,
            sys_pending: Mutex::new(VecDeque::new()),
            mic_pending: Mutex::new(VecDeque::new()),
        })
    }

    fn pop_sys(&self) -> [f32; 2] {
        self.sys_pending
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or([0.0, 0.0])
    }

    fn pop_mic(&self) -> f32 {
        self.mic_pending.lock().unwrap().pop_front().unwrap_or(0.0)
    }

    fn has_pending(&self) -> bool {
        let s_empty = self.sys_pending.lock().unwrap().is_empty();
        let m_empty = self.mic_pending.lock().unwrap().is_empty();
        !s_empty || !m_empty
    }

    pub fn push_system_stereo(&self, frames: &[[f32; 2]]) {
        if frames.is_empty() {
            return;
        }
        let mut q = self.sys_pending.lock().unwrap();
        q.extend(frames.iter().copied());
        while q.len() > MAX_QUEUE_FRAMES {
            q.pop_front();
        }
    }

    pub fn push_microphone(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        let mut q = self.mic_pending.lock().unwrap();
        q.extend(samples.iter().copied());
        while q.len() > MAX_QUEUE_FRAMES {
            q.pop_front();
        }
    }

    pub fn spawn_mixer_thread(mixer: Arc<Self>, running: Arc<AtomicBool>) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let mut next_tick = Instant::now();
            while running.load(Ordering::Acquire) || mixer.has_pending() {
                let pace_realtime = running.load(Ordering::Acquire);
                if pace_realtime {
                    let now = Instant::now();
                    if next_tick > now {
                        std::thread::sleep(next_tick - now);
                    } else {
                        next_tick = now;
                    }
                    next_tick += MIX_FRAME;
                }

                let mut batch = Vec::with_capacity(MIX_CHUNK_FRAMES);
                for _ in 0..MIX_CHUNK_FRAMES {
                    let [sys_l, sys_r] = mixer.pop_sys();
                    let mic = mixer.pop_mic() * SOURCE_GAIN;
                    batch.push([
                        soft_limit(sys_l * SOURCE_GAIN + mic),
                        soft_limit(sys_r * SOURCE_GAIN + mic),
                    ]);
                }
                mixer.output.emit_stereo_48k(&batch);

                if !pace_realtime && !mixer.has_pending() {
                    break;
                }
            }
        })
    }
}

/// Soft-clip toward ±1 instead of hard clamp, so mixed peaks don't brick-wall.
fn soft_limit(x: f32) -> f32 {
    const KNEE: f32 = 0.9;
    let mag = x.abs();
    if mag <= KNEE {
        return x;
    }
    let over = mag - KNEE;
    let shaped = KNEE + (1.0 - KNEE) * (over / (1.0 + over));
    x.signum() * shaped.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::soft_limit;
    use super::PLAYBACK_RATE;

    #[test]
    fn mix_chunk_is_ten_ms_at_playback_rate() {
        assert_eq!(super::MIX_CHUNK_FRAMES, PLAYBACK_RATE as usize / 100);
    }

    #[test]
    fn soft_limit_passes_quiet_audio() {
        assert!((soft_limit(0.2) - 0.2).abs() < f32::EPSILON);
        assert!((soft_limit(-0.5) + 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn soft_limit_stays_within_unity() {
        assert!(soft_limit(4.0) <= 1.0);
        assert!(soft_limit(-4.0) >= -1.0);
        assert!(soft_limit(1.5).abs() < 1.5);
    }
}
