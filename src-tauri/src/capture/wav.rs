use crate::capture::spectrum::SpectrumBuf;
use hound::{SampleFormat, WavSpec, WavWriter};
use ringbuf::{HeapRb, Producer};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Whisper and live meter still run at 16 kHz mono.
pub const SAMPLE_RATE: u32 = 16_000;
/// System + mic playback keeps SCK's native stereo rate.
pub const PLAYBACK_RATE: u32 = 48_000;

pub type RecordingWavWriter = WavWriter<BufWriter<File>>;

/// Dual-mono 16-bit PCM so WebViews don't play mic-only files on the left speaker only.
pub fn voice_wav_spec() -> WavSpec {
    WavSpec {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    }
}

/// True stereo 48 kHz — do not fold L/R before this is written.
pub fn system_wav_spec() -> WavSpec {
    WavSpec {
        channels: 2,
        sample_rate: PLAYBACK_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    }
}

pub fn create_wav_writer(path: &Path, spec: WavSpec) -> Result<RecordingWavWriter, String> {
    WavWriter::create(path, spec).map_err(|e| e.to_string())
}

fn f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

fn write_dual_mono(writer: &mut RecordingWavWriter, samples: &[f32]) {
    for &s in samples {
        let v = f32_to_i16(s);
        let _ = writer.write_sample(v);
        let _ = writer.write_sample(v);
    }
}

fn write_stereo_frames(writer: &mut RecordingWavWriter, frames: &[[f32; 2]]) {
    for &[l, r] in frames {
        let _ = writer.write_sample(f32_to_i16(l));
        let _ = writer.write_sample(f32_to_i16(r));
    }
}

/// 48 kHz stereo → 16 kHz mono for Whisper (average three mids). Simultaneous L/R only — not a time-smear.
#[derive(Default)]
pub struct Downmix48kTo16k {
    leftover: Vec<[f32; 2]>,
}

impl Downmix48kTo16k {
    pub fn push(&mut self, frames: &[[f32; 2]]) -> Vec<f32> {
        self.leftover.extend_from_slice(frames);
        let mut out = Vec::new();
        let mut i = 0;
        while i + 3 <= self.leftover.len() {
            let mut acc = 0.0f32;
            for k in 0..3 {
                let [l, r] = self.leftover[i + k];
                acc += (l + r) * 0.5;
            }
            out.push(acc / 3.0);
            i += 3;
        }
        if i > 0 {
            self.leftover.drain(..i);
        }
        out
    }
}

#[derive(Clone)]
pub struct MixOutput {
    pub producer: Arc<Mutex<Producer<f32, Arc<HeapRb<f32>>>>>,
    pub rms_sender: tokio::sync::watch::Sender<f32>,
    pub spectrum: Arc<Mutex<SpectrumBuf>>,
    pub wav: Arc<Mutex<Option<RecordingWavWriter>>>,
    pub whisper_downmix: Arc<Mutex<Downmix48kTo16k>>,
}

impl MixOutput {
    fn meter_mono(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        let rms = {
            let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
            (sum_sq / samples.len() as f32).sqrt()
        };
        let _ = self.rms_sender.send(rms);
        if let Ok(mut spectrum) = self.spectrum.lock() {
            spectrum.push(samples);
        }
        if let Ok(mut producer) = self.producer.lock() {
            let _ = producer.push_slice(samples);
        }
    }

    /// Mic-only path: 16 kHz mono samples, stored as dual-mono WAV.
    pub fn emit_mono_16k(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        if let Ok(mut slot) = self.wav.lock() {
            if let Some(writer) = slot.as_mut() {
                write_dual_mono(writer, samples);
            }
        }
        self.meter_mono(samples);
    }

    /// System + mic path: 48 kHz stereo frames for the file; Whisper gets 16 kHz mono.
    pub fn emit_stereo_48k(&self, frames: &[[f32; 2]]) {
        if frames.is_empty() {
            return;
        }
        if let Ok(mut slot) = self.wav.lock() {
            if let Some(writer) = slot.as_mut() {
                write_stereo_frames(writer, frames);
            }
        }
        let mono = {
            let mut d = self.whisper_downmix.lock().expect("downmix mutex poisoned");
            d.push(frames)
        };
        if !mono.is_empty() {
            self.meter_mono(&mono);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{f32_to_i16, Downmix48kTo16k};

    #[test]
    fn f32_to_i16_maps_unity() {
        assert_eq!(f32_to_i16(0.0), 0);
        assert_eq!(f32_to_i16(1.0), i16::MAX);
        assert_eq!(f32_to_i16(-1.0), -i16::MAX);
        assert_eq!(f32_to_i16(2.0), i16::MAX);
    }

    #[test]
    fn downmix_keeps_simultaneous_channels() {
        let mut d = Downmix48kTo16k::default();
        // Three frames: left=1, right=0 → mid 0.5, averaged over 3 → 0.5
        let frames = [[1.0, 0.0], [1.0, 0.0], [1.0, 0.0]];
        let out = d.push(&frames);
        assert_eq!(out.len(), 1);
        assert!((out[0] - 0.5).abs() < 1e-5);
    }
}
