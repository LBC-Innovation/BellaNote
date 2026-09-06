//! System audio via ScreenCaptureKit (display audio only). Microphone is always `cpal`.
//! Capture stays stereo at 48 kHz so playback is not a folded-mono mix.

use crate::capture::mic::StreamingResampler;
use crate::capture::mixer::SystemMicMixer;
use crate::capture::wav::PLAYBACK_RATE;
use anyhow::{Context, Result};
use screencapturekit::async_api::AsyncSCShareableContent;
use screencapturekit::cm::CMSampleBuffer;
use screencapturekit::stream::configuration::audio::{AudioChannelCount, AudioSampleRate};
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;
use screencapturekit::stream::output_trait::SCStreamOutputTrait;
use screencapturekit::stream::output_type::SCStreamOutputType;
use screencapturekit::stream::sc_stream::SCStream;
use std::sync::{Arc, Mutex};

fn read_f32(data: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn read_i16(data: &[u8], off: usize) -> f32 {
    i16::from_le_bytes([data[off], data[off + 1]]) as f32 / 32768.0
}

/// Interleaved or planar stereo (or mono duplicated). Never averages L+R into one stream.
fn pcm_f32_stereo_from_sample_buffer(sample: &CMSampleBuffer) -> (Vec<[f32; 2]>, u32) {
    let Some(fd) = sample.format_description() else {
        return (Vec::new(), PLAYBACK_RATE);
    };
    if !fd.is_audio() {
        return (Vec::new(), PLAYBACK_RATE);
    }
    let ch = fd.audio_channel_count().unwrap_or(2).max(1) as usize;
    let is_float = fd.audio_is_float();
    let bits = fd.audio_bits_per_channel().unwrap_or(16);
    let rate = fd.audio_sample_rate().unwrap_or(PLAYBACK_RATE as f64).round() as u32;
    let Some(list) = sample.audio_buffer_list() else {
        return (Vec::new(), rate);
    };

    if list.num_buffers() == 1 {
        let Some(b0) = list.get(0) else {
            return (Vec::new(), rate);
        };
        let data = b0.data();
        if data.is_empty() {
            return (Vec::new(), rate);
        }
        let mut out = Vec::new();
        if is_float {
            let step = 4 * ch;
            if step == 0 || data.len() < step {
                return (Vec::new(), rate);
            }
            for frame in data.chunks_exact(step) {
                let l = read_f32(frame, 0);
                let r = if ch >= 2 { read_f32(frame, 4) } else { l };
                out.push([l, r]);
            }
        } else if bits == 16 {
            let step = 2 * ch;
            if step == 0 || data.len() < step {
                return (Vec::new(), rate);
            }
            for frame in data.chunks_exact(step) {
                let l = read_i16(frame, 0);
                let r = if ch >= 2 { read_i16(frame, 2) } else { l };
                out.push([l, r]);
            }
        }
        return (out, rate);
    }

    let mut frames: Option<usize> = None;
    let mut per_ch: Vec<&[u8]> = Vec::new();
    for i in 0..list.num_buffers() {
        let Some(buf) = list.get(i) else {
            return (Vec::new(), rate);
        };
        let data = buf.data();
        let n = if is_float {
            data.len() / 4
        } else if bits == 16 {
            data.len() / 2
        } else {
            0
        };
        frames = Some(frames.map(|f| f.min(n)).unwrap_or(n));
        per_ch.push(data);
    }
    let Some(nf) = frames else {
        return (Vec::new(), rate);
    };
    if nf == 0 || per_ch.is_empty() {
        return (Vec::new(), rate);
    }
    let mut out = Vec::with_capacity(nf);
    let sample_at = |data: &[u8], i: usize| -> f32 {
        if is_float {
            read_f32(data, i * 4)
        } else {
            read_i16(data, i * 2)
        }
    };
    for i in 0..nf {
        let l = sample_at(per_ch[0], i);
        let r = if per_ch.len() >= 2 {
            sample_at(per_ch[1], i)
        } else {
            l
        };
        out.push([l, r]);
    }
    (out, rate)
}

struct StereoRateConverter {
    in_rate: u32,
    left: StreamingResampler,
    right: StreamingResampler,
}

impl StereoRateConverter {
    fn new(in_rate: u32) -> Self {
        Self {
            in_rate,
            left: StreamingResampler::new(in_rate, PLAYBACK_RATE),
            right: StreamingResampler::new(in_rate, PLAYBACK_RATE),
        }
    }

    fn push(&mut self, frames: &[[f32; 2]], in_rate: u32) -> Vec<[f32; 2]> {
        if in_rate != self.in_rate && in_rate > 0 {
            *self = Self::new(in_rate);
        }
        if frames.is_empty() {
            return Vec::new();
        }
        let mut ls = Vec::with_capacity(frames.len());
        let mut rs = Vec::with_capacity(frames.len());
        for &[l, r] in frames {
            ls.push(l);
            rs.push(r);
        }
        let lo = self.left.push_mono(&ls);
        let ro = self.right.push_mono(&rs);
        let n = lo.len().min(ro.len());
        lo.into_iter()
            .zip(ro)
            .take(n)
            .map(|(l, r)| [l, r])
            .collect()
    }
}

struct SystemAudioHandler {
    mixer: Arc<SystemMicMixer>,
    rate: Mutex<StereoRateConverter>,
}

impl SCStreamOutputTrait for SystemAudioHandler {
    fn did_output_sample_buffer(&self, sample_buffer: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Audio {
            return;
        }
        let (frames, rate) = pcm_f32_stereo_from_sample_buffer(&sample_buffer);
        if frames.is_empty() {
            return;
        }
        let resampled = {
            let mut conv = self.rate.lock().expect("sck rate mutex poisoned");
            conv.push(&frames, rate)
        };
        if !resampled.is_empty() {
            self.mixer.push_system_stereo(&resampled);
        }
    }
}

pub struct SystemAudioSession {
    stream: SCStream,
}

impl SystemAudioSession {
    pub async fn new(mixer: Arc<SystemMicMixer>) -> Result<Self> {
        let content = AsyncSCShareableContent::get()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let displays = content.displays();
        let display = displays.first().context("No display found")?;
        let filter = SCContentFilter::create()
            .with_display(display)
            .with_excluding_windows(&[])
            .build();

        let mut config = SCStreamConfiguration::new();
        config.set_captures_audio(true);
        config.set_excludes_current_process_audio(false);
        config.set_sample_rate(AudioSampleRate::Rate48000);
        config.set_channel_count(AudioChannelCount::Stereo);

        let mut stream = SCStream::new(&filter, &config);
        stream
            .add_output_handler(
                SystemAudioHandler {
                    mixer,
                    rate: Mutex::new(StereoRateConverter::new(PLAYBACK_RATE)),
                },
                SCStreamOutputType::Audio,
            )
            .context("failed to add system audio output handler")?;
        Ok(Self { stream })
    }

    pub fn start(&self) -> Result<()> {
        self.stream
            .start_capture()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.stream
            .stop_capture()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }
}
